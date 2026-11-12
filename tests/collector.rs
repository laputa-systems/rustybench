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

#[derive(Deserialize)]
struct Baseline { benchmarks: Vec<Record>, measured_runs: u32 }

#[derive(Deserialize)]
struct Record { name: String, run_median_ns: Option<Vec<u64>> }

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
    assert_eq!(file.benchmarks.len(), 4);
    assert!(file.benchmarks.iter().all(|record| record.run_median_ns.as_ref().unwrap().len() == 2));
    assert!(file.benchmarks.iter().any(|record| record.name.ends_with("a::Widget")));
    assert!(file.benchmarks.iter().any(|record| record.name.ends_with("b::Widget")));

    let script = format!("{exe} --bench --format json --sample-count 1 --sample-size 1 'colliding' && {exe} --bench --format json --sample-count 1 --sample-size 1 'stable$'", exe = quote(executable.to_str().unwrap()));
    success(collect().args(["sh", "-c", &script, "producer"]).output().unwrap());
    let file: Baseline = deser_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(file.benchmarks.len(), 3);

    let previous = fs::read(&path).unwrap();
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
