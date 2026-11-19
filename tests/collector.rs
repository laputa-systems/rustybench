use std::{fs, path::PathBuf, process::{Command, Output}};
use deser::Deserialize;

mod a { pub struct Widget; }
mod b { pub struct Widget; }

#[rustybench::bench(types = [a::Widget, b::Widget])]
fn colliding<T>() { rustybench::black_box(std::mem::size_of::<T>()); }

#[rustybench::bench]
fn stable() { rustybench::black_box(1); }

#[rustybench::bench]
fn conditional(bencher: rustybench::Bencher) {
    if std::env::var_os("RUSTYBENCH_FIXTURE_OMIT").is_none() {
        bencher.bench(|| rustybench::black_box(1));
    }
}

#[rustybench::bench_group]
mod nested {
    #[rustybench::bench(args = ["a[1]", "b(2)"], threads = [1, 2])]
    fn inputs(value: &str) { rustybench::black_box(value); }
}

#[rustybench::bench_group(ignore)]
mod ignored {
    #[rustybench::bench]
    fn excluded() { panic!("ignored benchmark executed"); }
}

#[derive(Deserialize)]
struct Baseline { benchmarks: Vec<Record>, measured_runs: u32 }

#[derive(Deserialize)]
struct Record { name: String, run_median_ns: Option<Vec<u64>> }

#[derive(Deserialize)]
struct Report { benchmarks: Vec<Record> }

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}

fn success(output: Output) -> Vec<u8> {
    assert!(output.status.success(), "{output:?}");
    output.stdout
}

fn quote(value: &str) -> String { format!("'{}'", value.replace('\'', "'\\''")) }

fn main() {
    if std::env::var_os("RUSTYBENCH_FIXTURE_CHILD").is_some() {
        rustybench::Rustybench::from_args().main();
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scratch = Scratch(root.join(format!(".work/tmp/collector-{}", std::process::id())));
    fs::create_dir_all(&scratch.0).unwrap();
    let executable = std::env::current_exe().unwrap();
    let path = scratch.0.join("baseline.json");
    let collect = || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rustybench"));
        command.args(["baseline", "--quiet", "--warmup", "0", "--runs", "2", "--sample-count", "1", "--sample-size", "1", "--baseline"])
            .arg(&path).arg("--").env("RUSTYBENCH_FIXTURE_CHILD", "1");
        command
    };
    success(collect().arg(&executable).output().unwrap());
    let file: Baseline = deser_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(file.measured_runs, 2);
    assert_eq!(file.benchmarks.len(), 8);
    assert!(file.benchmarks.iter().all(|record| record.run_median_ns.as_ref().unwrap().len() == 2));
    assert!(file.benchmarks.iter().any(|record| record.name.ends_with("a::Widget")));
    assert!(file.benchmarks.iter().any(|record| record.name.ends_with("b::Widget")));

    let listing = success(Command::new(&executable).env("RUSTYBENCH_FIXTURE_CHILD", "1")
        .args(["--list", "--format", "terse"]).output().unwrap());
    let listing = String::from_utf8(listing).unwrap();
    let names: Vec<_> = listing.lines().filter_map(|line| line.strip_suffix(": benchmark")).collect();
    assert_eq!(names.len(), 8, "{listing}");
    assert!(names.iter().any(|name| name.ends_with("inputs::a[1]::t=1")));
    assert!(names.iter().any(|name| name.ends_with("inputs::b(2)::t=2")));
    assert!(!listing.contains("excluded"));
    for name in names {
        let json = success(Command::new(&executable).env("RUSTYBENCH_FIXTURE_CHILD", "1")
            .args(["--bench", "--format", "json", "--sample-count", "1", "--sample-size", "1", "--exact", name]).output().unwrap());
        let report: Report = deser_json::from_str(std::str::from_utf8(&json).unwrap()).unwrap();
        assert_eq!(report.benchmarks.len(), 1, "{name}");
        assert_eq!(report.benchmarks[0].name, name);
    }

    let human = success(Command::new(&executable).env("RUSTYBENCH_FIXTURE_CHILD", "1")
        .args(["--bench", "--format", "human", "--sample-count", "2", "--sample-size", "3", "--exact", "collector::stable"]).output().unwrap());
    let human = String::from_utf8(human).unwrap();
    let header = human.lines().find(|line| line.contains("fastest") && line.contains("median")).unwrap();
    assert!(header.contains("slowest") && header.contains("mean") && header.contains("samples") && header.contains("iters"), "{human}");
    let row = human.lines().find(|line| line.contains("stable")).unwrap();
    let columns: Vec<_> = row.split('│').map(str::trim).collect();
    assert_eq!(columns.len(), 6, "{human}");
    assert_eq!(&columns[4..], &["2", "6"], "{human}");

    let script = format!("{exe} --bench --format json --sample-count 1 --sample-size 1 'colliding' && {exe} --bench --format json --sample-count 1 --sample-size 1 'stable$'", exe = quote(executable.to_str().unwrap()));
    success(collect().args(["sh", "-c", &script, "producer"]).output().unwrap());
    let file: Baseline = deser_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(file.benchmarks.len(), 3);

    let previous = fs::read(&path).unwrap();
    let budget = scratch.0.join("budget.json");
    fs::write(&budget, br#"{"schema":1,"host":"h","platform":"p","arch":"a","mode":"m","warmup_runs":0,"measured_runs":1,"wall_ns":1,"warmup_wall_ns":0,"measured_wall_ns":1,"suite_wall_ns":[1],"benchmarks":[{"name":"x","median_ns":10,"alloc_count":1,"alloc_bytes":2,"max_alloc_count":1,"max_alloc_bytes":2}]}"#).unwrap();
    let candidate = scratch.0.join("candidate.json");
    fs::write(&candidate, fs::read(&budget).unwrap()).unwrap();
    let tool = env!("CARGO_BIN_EXE_rustybench");
    success(Command::new(tool).arg("check").arg(&budget).arg(&candidate).output().unwrap());
    let over = String::from_utf8(fs::read(&candidate).unwrap()).unwrap().replace("\"median_ns\":10", "\"median_ns\":11");
    fs::write(&candidate, over).unwrap();
    let output = Command::new(tool).arg("check").arg(&budget).arg(&candidate).output().unwrap();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("median_ns is 11, exceeding budget 10"), "{output:?}");
    let output = Command::new(tool).arg("diff").arg(&budget).arg(&candidate).output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("+10.00%"), "{output:?}");
    let output = collect().arg(&executable).env("RUSTYBENCH_FIXTURE_OMIT", "1").output().unwrap();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("no benchmark function registered"));
    assert_eq!(fs::read(&path).unwrap(), previous);

    for flag in ["--sample-count", "--sample-size", "--max-time"] {
        let output = Command::new(&executable).env("RUSTYBENCH_FIXTURE_CHILD", "1")
            .args(["--bench", flag, "0", "--format", "json"]).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        success(Command::new(&executable).env("RUSTYBENCH_FIXTURE_CHILD", "1")
            .args(["--test", flag, "0"]).output().unwrap());
    }
    println!("collector acceptance passed");
}
