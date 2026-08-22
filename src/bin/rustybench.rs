use std::{
    collections::BTreeMap,
    env,
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use lexopt::{Parser, prelude::*};
use miniserde::{Deserialize, Serialize};

const BASELINE_SCHEMA: u32 = 1;
const SYSCALL_SCHEMA: u32 = 1;
const SYSCALL_IMAGE: &str = "benchmark-syscalls:local";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BenchmarkRecord {
    name: String,
    median_ns: u64,
    alloc_count: u64,
    alloc_bytes: u64,
    max_alloc_count: u64,
    max_alloc_bytes: u64,
    sample_count: u32,
    iter_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkReport {
    schema: u32,
    benchmarks: Vec<BenchmarkRecord>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BaselineRecord {
    name: String,
    median_ns: u64,
    alloc_count: u64,
    alloc_bytes: u64,
    max_alloc_count: u64,
    max_alloc_bytes: u64,
    /// Per-run timing medians are optional so schema-1 baselines written before
    /// instability reporting remains readable.
    run_median_ns: Option<Vec<u64>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BaselineFile {
    schema: u32,
    host: String,
    platform: String,
    arch: String,
    mode: String,
    warmup_runs: u32,
    measured_runs: u32,
    sample_count: Option<u32>,
    sample_size: Option<u32>,
    wall_ns: u64,
    warmup_wall_ns: u64,
    measured_wall_ns: u64,
    suite_wall_ns: Vec<u64>,
    benchmarks: Vec<BaselineRecord>,
}

#[derive(Debug, Default)]
struct BaselineOptions {
    root: Option<PathBuf>,
    baseline: Option<PathBuf>,
    variant: Option<String>,
    quiet: bool,
    print_path: bool,
    fast: bool,
    warmup: Option<u32>,
    runs: Option<u32>,
    sample_count: Option<u32>,
    sample_size: Option<u32>,
    command: Vec<OsString>,
}

#[derive(Debug, Default)]
struct SyscallOptions {
    root: Option<PathBuf>,
    format: SyscallFormat,
    sample_count: u32,
    sample_size: u32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum SyscallFormat {
    #[default]
    Human,
    Json,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
struct SyscallReport {
    schema: u32,
    /// This report contains strace diagnostics, not benchmark measurements.
    diagnostic: bool,
    /// Syscall collection is deliberately outside Rustybench's timing contract.
    timing: bool,
    benchmarks: Vec<SyscallBenchmark>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
struct SyscallBenchmark {
    name: String,
    marker_status: String,
    calls: u64,
    errors: u64,
    syscalls: Vec<SyscallCount>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
struct SyscallCount {
    name: String,
    calls: u64,
    errors: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedSyscalls {
    marker_status: &'static str,
    calls: u64,
    errors: u64,
    syscalls: Vec<SyscallCount>,
}

fn main() {
    let mut arguments = std::env::args_os();
    let program = arguments
        .next()
        .unwrap_or_else(|| OsString::from("rustybench"));
    let Some(command) = arguments.next() else {
        print_help(&program);
        return;
    };

    let result = match command.to_str() {
        Some("baseline") => baseline(arguments.collect()),
        Some("diff") => diff(arguments.collect()),
        Some("syscalls") => syscalls(arguments.collect()),
        Some("help") | Some("--help") | Some("-h") => {
            print_help(&program);
            Ok(())
        }
        _ => Err(format!(
            "unknown subcommand {command:?}; expected baseline, diff, or syscalls"
        )),
    };

    if let Err(error) = result {
        eprintln!("rustybench: {error}");
        std::process::exit(2);
    }
}

fn print_help(program: &OsStr) {
    eprintln!(
        "Usage: {} <baseline|diff|syscalls> [OPTIONS]\n\n\
         baseline [OPTIONS] -- COMMAND [ARGS...]\n\
         baseline runs COMMAND repeatedly, collects rustybench JSON, and writes a JSON baseline.\n\
         diff BASELINE CANDIDATE\n\
         syscalls [--root PATH] [--format human|json]\n\
                                  Linux Docker/strace diagnostic (not timing)\n\n\
         baseline options:\n\
           --root PATH             Run COMMAND in PATH\n\
           --baseline PATH         Baseline JSON path\n\
           --variant NAME          Suffix the host-derived path\n\
           --warmup N              Warmup runs (default 1)\n\
           --runs N                Measured runs (default 3)\n\
           --sample-count N        Pass through to the benchmark executable\n\
           --sample-size N         Pass through to the benchmark executable\n\
           --fast                   One measured run with one sample\n\
           --quiet                  Do not print the comparison table\n\
           --print-path             Print the selected path and exit\n\n\
         syscall options:\n\
           --format human|json      Human table (default) or schema-1 JSON;\n\
                                    JSON is diagnostic-only and timing is false",
        program.to_string_lossy()
    );
}

fn baseline(arguments: Vec<OsString>) -> Result<(), String> {
    let (option_arguments, command) = split_command(arguments);
    let options = parse_baseline_options(option_arguments, command)?;
    let root = options
        .root
        .clone()
        .unwrap_or_else(|| env::current_dir().expect("current directory is available"));
    let path = options.baseline.clone().unwrap_or_else(|| {
        let (host, _, _) = host_info();
        let suffix = options
            .variant
            .as_deref()
            .map(|value| format!("-{value}"))
            .unwrap_or_default();
        root.join("benches")
            .join(format!("{host}{suffix}-baseline.json"))
    });

    if options.print_path {
        println!("{}", path.display());
        return Ok(());
    }

    let command = if options.command.is_empty() {
        vec![
            OsString::from("cargo"),
            OsString::from("bench"),
            OsString::from("--bench"),
            OsString::from("bench"),
        ]
    } else {
        options.command.clone()
    };

    let (warmup_runs, measured_runs, sample_count, sample_size) = if options.fast {
        (0, 1, Some(1), Some(1))
    } else {
        (
            options.warmup.unwrap_or(1),
            options.runs.unwrap_or(3),
            options.sample_count,
            options.sample_size,
        )
    };
    if measured_runs == 0 {
        return Err("--runs must be greater than zero".to_owned());
    }

    let mut warmup_wall_ns = 0u64;
    for _ in 0..warmup_runs {
        let started = Instant::now();
        run_benchmark(&root, &command, sample_count, sample_size)?;
        warmup_wall_ns = warmup_wall_ns.saturating_add(duration_ns(started.elapsed()));
    }

    let mut runs = Vec::with_capacity(measured_runs as usize);
    let mut suite_wall_ns = Vec::with_capacity(measured_runs as usize);
    for _ in 0..measured_runs {
        let started = Instant::now();
        let report = run_benchmark(&root, &command, sample_count, sample_size)?;
        suite_wall_ns.push(duration_ns(started.elapsed()));
        runs.push(report);
    }

    let current = aggregate_reports(&runs);
    if current.is_empty() {
        return Err("benchmark command produced no rustybench JSON records".to_owned());
    }

    let previous = if path.exists() {
        read_baseline(&path)?
    } else {
        Vec::new()
    };
    if !options.quiet {
        print_comparison(&previous, &current, path.display());
    }

    let (host, platform, arch) = host_info();
    let file = BaselineFile {
        schema: BASELINE_SCHEMA,
        host,
        platform,
        arch,
        mode: if options.fast { "fast" } else { "normal" }.to_owned(),
        warmup_runs,
        measured_runs,
        sample_count,
        sample_size,
        wall_ns: warmup_wall_ns.saturating_add(suite_wall_ns.iter().copied().sum()),
        warmup_wall_ns,
        measured_wall_ns: suite_wall_ns.iter().copied().sum(),
        suite_wall_ns,
        benchmarks: current,
    };
    write_json_atomic(&path, &file)
}

fn split_command(arguments: Vec<OsString>) -> (Vec<OsString>, Vec<OsString>) {
    match arguments.iter().position(|value| value == "--") {
        Some(index) => (arguments[..index].to_vec(), arguments[index + 1..].to_vec()),
        None => (arguments, Vec::new()),
    }
}

fn parse_baseline_options(
    arguments: Vec<OsString>,
    command: Vec<OsString>,
) -> Result<BaselineOptions, String> {
    let mut options = BaselineOptions {
        command,
        ..BaselineOptions::default()
    };
    let mut parser = Parser::from_args(arguments);
    while let Some(argument) = parser.next().map_err(|error| error.to_string())? {
        match argument {
            Long("root") => {
                options.root = Some(PathBuf::from(parser.value().map_err(|e| e.to_string())?))
            }
            Long("baseline") => {
                options.baseline = Some(PathBuf::from(parser.value().map_err(|e| e.to_string())?));
            }
            Long("variant") => options.variant = Some(value(parser.value())?),
            Long("quiet") => options.quiet = true,
            Long("print-path") => options.print_path = true,
            Long("fast") => options.fast = true,
            Long("warmup") => options.warmup = Some(parse_value(&mut parser, "warmup")?),
            Long("runs") => options.runs = Some(parse_value(&mut parser, "runs")?),
            Long("sample-count") => {
                options.sample_count = Some(parse_value(&mut parser, "sample-count")?);
            }
            Long("sample-size") => {
                options.sample_size = Some(parse_value(&mut parser, "sample-size")?);
            }
            Value(value) => return Err(format!("unexpected argument {value:?}")),
            _ => return Err(argument.unexpected().to_string()),
        }
    }
    Ok(options)
}

fn parse_value<T: std::str::FromStr>(parser: &mut Parser, name: &str) -> Result<T, String>
where
    T::Err: std::fmt::Display,
{
    let value = parser.value().map_err(|error| error.to_string())?;
    value
        .to_str()
        .ok_or_else(|| format!("--{name} value is not valid Unicode"))?
        .parse()
        .map_err(|error| format!("invalid --{name} value: {error}"))
}

fn value(value: Result<OsString, lexopt::Error>) -> Result<String, String> {
    value
        .map_err(|error| error.to_string())?
        .into_string()
        .map_err(|_| "argument is not valid Unicode".to_owned())
}

fn run_benchmark(
    root: &Path,
    command: &[OsString],
    sample_count: Option<u32>,
    sample_size: Option<u32>,
) -> Result<BenchmarkReport, String> {
    let mut command = command.to_vec();
    let separator = command.iter().position(|value| value == "--");
    let insert_at = if let Some(index) = separator {
        index + 1
    } else {
        command.push(OsString::from("--"));
        command.len()
    };
    command.insert(insert_at, OsString::from("--bench"));
    command.insert(insert_at + 1, OsString::from("--format"));
    command.insert(insert_at + 2, OsString::from("json"));
    let mut next = insert_at + 3;
    if let Some(value) = sample_count {
        command.insert(next, OsString::from("--sample-count"));
        command.insert(next + 1, value.to_string().into());
        next += 2;
    }
    if let Some(value) = sample_size {
        command.insert(next, OsString::from("--sample-size"));
        command.insert(next + 1, value.to_string().into());
    }

    let mut child = Command::new(&command[0]);
    child.args(&command[1..]).current_dir(root);
    if let Some(flags) = env::var_os("RUSTYBENCH_BENCH_RUSTFLAGS") {
        child.env("RUSTFLAGS", flags);
    }
    let output = child
        .output()
        .map_err(|error| format!("could not run {:?}: {error}", command[0]))?;
    if !output.status.success() {
        return Err(format_command_failure(&command, &output));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .rev()
        .find_map(|line| {
            let report = miniserde::json::from_str::<BenchmarkReport>(line.trim()).ok()?;
            (report.schema == BASELINE_SCHEMA).then_some(report)
        })
        .ok_or_else(|| {
            format!(
                "could not parse benchmark JSON from {:?}\n{}",
                command, stdout
            )
        })
}

fn format_command_failure(command: &[OsString], output: &std::process::Output) -> String {
    format!(
        "command {:?} failed with {}\n{}{}",
        command,
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn aggregate_reports(reports: &[BenchmarkReport]) -> Vec<BaselineRecord> {
    let mut values: BTreeMap<String, Vec<BenchmarkRecord>> = BTreeMap::new();
    for report in reports {
        for record in &report.benchmarks {
            values
                .entry(record.name.clone())
                .or_default()
                .push(record.clone());
        }
    }
    values
        .into_iter()
        .map(|(name, records)| BaselineRecord {
            name,
            median_ns: median(records.iter().map(|record| record.median_ns)),
            alloc_count: median(records.iter().map(|record| record.alloc_count)),
            alloc_bytes: median(records.iter().map(|record| record.alloc_bytes)),
            max_alloc_count: median(records.iter().map(|record| record.max_alloc_count)),
            max_alloc_bytes: median(records.iter().map(|record| record.max_alloc_bytes)),
            run_median_ns: Some(records.iter().map(|record| record.median_ns).collect()),
        })
        .collect()
}

fn median(values: impl Iterator<Item = u64>) -> u64 {
    let mut values: Vec<_> = values.collect();
    values.sort_unstable();
    values[values.len() / 2]
}

fn diff(arguments: Vec<OsString>) -> Result<(), String> {
    let mut parser = Parser::from_args(arguments);
    let mut paths = Vec::new();
    while let Some(argument) = parser.next().map_err(|error| error.to_string())? {
        match argument {
            Value(value) => paths.push(PathBuf::from(value)),
            _ => return Err(argument.unexpected().to_string()),
        }
    }
    if paths.len() != 2 {
        return Err("diff expects BASELINE and CANDIDATE".to_owned());
    }
    let baseline = read_baseline(&paths[0])?;
    let candidate = read_baseline(&paths[1])?;
    print_comparison(
        &baseline,
        &candidate,
        format!("{} → {}", paths[0].display(), paths[1].display()),
    );
    Ok(())
}

fn read_baseline(path: &Path) -> Result<Vec<BaselineRecord>, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    parse_baseline(&contents, path)
}

fn parse_baseline(contents: &str, path: &Path) -> Result<Vec<BaselineRecord>, String> {
    let baseline: BaselineFile = miniserde::json::from_str(contents)
        .map_err(|error| format!("invalid baseline JSON {}: {error}", path.display()))?;
    if baseline.schema != BASELINE_SCHEMA {
        return Err(format!("unsupported baseline schema {}", baseline.schema));
    }
    Ok(baseline.benchmarks)
}

fn write_json_atomic(path: &Path, value: &BaselineFile) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
    }
    let temporary = path.with_file_name(format!(
        ".{}.{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id()
    ));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| format!("could not create {}: {error}", temporary.display()))?;
    let serialized = miniserde::json::to_string(value);
    if let Err(error) = file
        .write_all(serialized.as_bytes())
        .and_then(|_| file.flush())
    {
        let _ = fs::remove_file(&temporary);
        return Err(format!("could not write {}: {error}", path.display()));
    }
    drop(file);
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("could not replace {}: {error}", path.display()));
    }
    Ok(())
}

fn print_comparison(
    previous: &[BaselineRecord],
    current: &[BaselineRecord],
    title: impl std::fmt::Display,
) {
    print!(
        "{}",
        render_comparison(previous, current, title, comparison_width())
    );
}

fn comparison_width() -> usize {
    env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|&value: &usize| value >= 48)
        .unwrap_or(140)
}

fn render_comparison(
    previous: &[BaselineRecord],
    current: &[BaselineRecord],
    title: impl std::fmt::Display,
    width: usize,
) -> String {
    const SEPARATORS: usize = 3;
    let metric_width = if width >= 100 {
        32
    } else if width >= 60 {
        14
    } else {
        10
    };
    let name_width = width.saturating_sub(metric_width * 3 + SEPARATORS).max(8);
    let line_width = name_width + metric_width * 3 + SEPARATORS;
    let previous: BTreeMap<_, _> = previous
        .iter()
        .map(|record| (&record.name, record))
        .collect();
    let current: BTreeMap<_, _> = current
        .iter()
        .map(|record| (&record.name, record))
        .collect();
    let names = previous
        .keys()
        .chain(current.keys())
        .map(|name| (*name).clone())
        .collect::<std::collections::BTreeSet<_>>();

    let mut output = String::new();
    output.push_str(&format!("{title}\n"));
    output.push_str(&format!(
        "{:<name_width$} {:>metric_width$} {:>metric_width$} {:>metric_width$}\n",
        "benchmark", "time", "memory", "allocs/op"
    ));
    output.push_str(&format!("{}\n", "-".repeat(line_width)));

    for name in names {
        let (time, memory, allocs) = match (previous.get(&name), current.get(&name)) {
            (Some(previous), Some(current)) => (
                format_metric_with_spread(
                    current.median_ns,
                    Some(previous.median_ns),
                    current.run_median_ns.as_deref(),
                ),
                format_metric(current.alloc_bytes, Some(previous.alloc_bytes), "B"),
                format_metric(current.alloc_count, Some(previous.alloc_count), ""),
            ),
            (None, Some(current)) => (
                format_metric_with_spread(
                    current.median_ns,
                    None,
                    current.run_median_ns.as_deref(),
                ),
                format_metric(current.alloc_bytes, None, "B"),
                format_metric(current.alloc_count, None, ""),
            ),
            (Some(_), None) => (
                "removed".to_owned(),
                "removed".to_owned(),
                "removed".to_owned(),
            ),
            (None, None) => unreachable!(),
        };
        output.push_str(&format!(
            "{:<name_width$} {:>metric_width$} {:>metric_width$} {:>metric_width$}\n",
            truncate_cell(&name, name_width),
            truncate_cell(&time, metric_width),
            truncate_cell(&memory, metric_width),
            truncate_cell(&allocs, metric_width),
        ));
    }
    output
}

fn truncate_cell(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        return value.to_owned();
    }
    if width <= 1 {
        return "…".chars().take(width).collect();
    }
    let mut result: String = value.chars().take(width - 1).collect();
    result.push('…');
    result
}

fn format_metric_with_spread(current: u64, previous: Option<u64>, runs: Option<&[u64]>) -> String {
    let metric = format_metric(current, previous, "ns");
    let Some(runs) = runs.filter(|runs| runs.len() > 1) else {
        return metric;
    };
    format!("{metric} [{}]", format_spread(runs))
}

fn format_spread(values: &[u64]) -> String {
    let (Some(min), Some(max)) = (values.iter().min(), values.iter().max()) else {
        return "stable".to_owned();
    };
    if min == max {
        return "stable".to_owned();
    }
    format!("{}–{}", format_duration(*min), format_duration(*max))
}

fn format_metric(current: u64, previous: Option<u64>, unit: &str) -> String {
    let value = if unit == "ns" {
        format_duration(current)
    } else if unit == "B" {
        format_bytes(current)
    } else {
        format!("{current} allocs")
    };
    let Some(previous) = previous else {
        return format!("{value} (new)");
    };
    if previous == 0 {
        return format!("{value} ({})", if current == 0 { "0.00%" } else { "new" });
    }
    let change = (current as f64 - previous as f64) / previous as f64 * 100.0;
    format!("{value} ({change:+.2}%)")
}

fn format_duration(value: u64) -> String {
    if value < 1_000 {
        format!("{value} ns")
    } else if value < 1_000_000 {
        format!("{:.2} µs", value as f64 / 1_000.0)
    } else if value < 1_000_000_000 {
        format!("{:.2} ms", value as f64 / 1_000_000.0)
    } else {
        format!("{:.2} s", value as f64 / 1_000_000_000.0)
    }
}

fn format_bytes(value: u64) -> String {
    if value < 1024 {
        format!("{value} B")
    } else if value < 1 << 20 {
        format!("{:.2} KB", value as f64 / 1024.0)
    } else {
        format!("{:.2} MB", value as f64 / (1 << 20) as f64)
    }
}

fn duration_ns(duration: Duration) -> u64 {
    duration.as_nanos().try_into().unwrap_or(u64::MAX)
}

fn host_info() -> (String, String, String) {
    let platform = env::consts::OS.to_owned();
    let arch = env::consts::ARCH.to_owned();
    let cpus = if cfg!(target_os = "macos") {
        command_output("sysctl", &["-n", "hw.ncpu"]).unwrap_or_else(|| "unknown".to_owned())
    } else {
        fs::read_to_string("/proc/cpuinfo")
            .map(|value| {
                value
                    .lines()
                    .filter(|line| line.starts_with("processor"))
                    .count()
                    .to_string()
            })
            .unwrap_or_else(|_| "unknown".to_owned())
    };
    let memory = if cfg!(target_os = "macos") {
        command_output("sysctl", &["-n", "hw.memsize"])
            .and_then(|value| value.parse::<u64>().ok())
            .map(|bytes| bytes.div_ceil(1 << 30).to_string())
            .unwrap_or_else(|| "unknown".to_owned())
    } else {
        fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|value| {
                value
                    .lines()
                    .find(|line| line.starts_with("MemTotal:"))?
                    .split_whitespace()
                    .nth(1)?
                    .parse::<u64>()
                    .ok()
            })
            .map(|kb| (kb * 1024).div_ceil(1 << 30).to_string())
            .unwrap_or_else(|| "unknown".to_owned())
    };
    (
        format!("{platform}-{arch}-{cpus}-{memory}gb"),
        platform,
        arch,
    )
}

fn command_output(program: &str, arguments: &[&str]) -> Option<String> {
    let output = Command::new(program).args(arguments).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        None
    }
}

fn syscalls(arguments: Vec<OsString>) -> Result<(), String> {
    if env::consts::OS != "linux" {
        return Err("syscalls is supported on Linux only".to_owned());
    }
    let mut options = SyscallOptions {
        sample_count: 10,
        sample_size: 100,
        ..SyscallOptions::default()
    };
    let mut parser = Parser::from_args(arguments);
    while let Some(argument) = parser.next().map_err(|error| error.to_string())? {
        match argument {
            Long("root") => {
                options.root = Some(PathBuf::from(parser.value().map_err(|e| e.to_string())?))
            }
            Long("sample-count") => {
                options.sample_count = parse_value(&mut parser, "sample-count")?
            }
            Long("sample-size") => options.sample_size = parse_value(&mut parser, "sample-size")?,
            Long("format") => {
                options.format = match value(parser.value())?.as_str() {
                    "human" => SyscallFormat::Human,
                    "json" => SyscallFormat::Json,
                    format => {
                        return Err(format!(
                            "invalid --format value {format:?}; expected human or json"
                        ));
                    }
                };
            }
            Value(value) => return Err(format!("unexpected argument {value:?}")),
            _ => return Err(argument.unexpected().to_string()),
        }
    }
    let root = options
        .root
        .unwrap_or_else(|| env::current_dir().expect("current directory is available"));
    run_syscall_diagnostic(
        &root,
        options.sample_count,
        options.sample_size,
        options.format,
    )
}

fn run_syscall_diagnostic(
    root: &Path,
    sample_count: u32,
    sample_size: u32,
    format: SyscallFormat,
) -> Result<(), String> {
    docker(root, &["build", "--quiet", "-t", SYSCALL_IMAGE, "."])?;
    let executable = docker_capture(
        root,
        "cargo bench --bench bench --no-run --message-format=json",
    )?
    .lines()
    .filter_map(|line| miniserde::json::from_str::<CargoArtifact>(line).ok())
    .find_map(|artifact| {
        if artifact.reason == "compiler-artifact" && artifact.target.name == "bench" {
            artifact.executable
        } else {
            None
        }
    })
    .ok_or_else(|| "could not find benchmark executable in cargo JSON".to_owned())?;
    let names = docker_capture(
        root,
        &format!("{} --bench --list", shell_quote(&executable)),
    )?
    .lines()
    .filter_map(|line| {
        line.strip_prefix("├─ ")
            .or_else(|| line.strip_prefix("╰─ "))
    })
    .map(ToOwned::to_owned)
    .collect::<std::collections::BTreeSet<_>>()
    .into_iter()
    .collect::<Vec<_>>();
    if names.is_empty() {
        return Err("benchmark executable listed no benchmarks".to_owned());
    }
    let mut benchmarks = Vec::with_capacity(names.len());
    for name in names {
        let command = format!(
            "rm -f /tmp/strace-events && SYSCALL_TRACE=1 strace -f -qq -o /tmp/strace-events {} --bench {} --sample-count {} --sample-size {} >/dev/null 2>&1 && cat /tmp/strace-events",
            shell_quote(&executable),
            shell_quote(&name),
            sample_count,
            sample_size
        );
        let trace = docker_capture(root, &command)?;
        let parsed = parse_syscalls(&trace);
        benchmarks.push(SyscallBenchmark {
            name: name.clone(),
            marker_status: parsed.marker_status.to_owned(),
            calls: parsed.calls,
            errors: parsed.errors,
            syscalls: parsed.syscalls.clone(),
        });
        if format == SyscallFormat::Human {
            println!(
                "{name} ({} iterations; diagnostic only; timing not measured)",
                sample_count as u64 * sample_size as u64
            );
            print_syscalls(&parsed);
            println!();
        }
    }
    if format == SyscallFormat::Json {
        let report = SyscallReport {
            schema: SYSCALL_SCHEMA,
            diagnostic: true,
            timing: false,
            benchmarks,
        };
        println!("{}", miniserde::json::to_string(&report));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct CargoArtifact {
    reason: String,
    target: CargoTarget,
    executable: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoTarget {
    name: String,
}

fn docker(root: &Path, arguments: &[&str]) -> Result<(), String> {
    let mut command = docker_base(root);
    command.args(arguments);
    command.stdout(Stdio::null());
    let status = command
        .status()
        .map_err(|error| format!("could not run docker: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("docker command failed: {status}"))
    }
}

fn docker_capture(root: &Path, script: &str) -> Result<String, String> {
    let mut command = docker_base(root);
    command.args(["sh", "-lc", script]);
    let output = command
        .output()
        .map_err(|error| format!("could not run docker: {error}"))?;
    if !output.status.success() {
        return Err(format_command_failure(&[OsString::from("docker")], &output));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn docker_base(root: &Path) -> Command {
    let mut command = Command::new("docker");
    command.args([
        "run",
        "--rm",
        "--cap-add=SYS_PTRACE",
        "--security-opt",
        "seccomp=unconfined",
        "-v",
    ]);
    command.arg(format!("{}:/workspace", root.display()));
    command.args(["-w", "/workspace", SYSCALL_IMAGE]);
    command
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn parse_syscalls(trace: &str) -> ParsedSyscalls {
    let mut totals: BTreeMap<String, (u64, u64)> = BTreeMap::new();
    let mut active = false;
    let mut saw_begin = false;
    let mut saw_end = false;
    for line in trace.lines() {
        if line.contains("prctl(PR_SET_NAME, \"BENCH_BEGIN\"") {
            saw_begin = true;
            active = true;
            continue;
        }
        if line.contains("prctl(PR_SET_NAME, \"BENCH_END\"") {
            saw_end = true;
            active = false;
            continue;
        }
        if !active || !line.contains(" = ") {
            continue;
        }
        let Some(name) = line
            .split_whitespace()
            .find(|value| value.contains('('))
            .and_then(|value| value.split('(').next())
        else {
            continue;
        };
        let entry = totals.entry(name.to_owned()).or_default();
        entry.0 += 1;
        if line.contains(" = -1") {
            entry.1 += 1;
        }
    }

    let marker_status = if !saw_begin {
        "missing-begin"
    } else if active || !saw_end {
        "missing-end"
    } else {
        "bounded"
    };
    let syscalls = totals
        .into_iter()
        .map(|(name, (calls, errors))| SyscallCount {
            name,
            calls,
            errors,
        })
        .collect::<Vec<_>>();
    let calls = syscalls
        .iter()
        .map(|syscall| syscall.calls)
        .sum();
    let errors = syscalls
        .iter()
        .map(|syscall| syscall.errors)
        .sum();
    ParsedSyscalls {
        marker_status,
        calls,
        errors,
        syscalls,
    }
}

fn print_syscalls(parsed: &ParsedSyscalls) {
    println!("  marker bounds: {}", parsed.marker_status);
    println!("  syscall                         calls     errors");
    for syscall in &parsed.syscalls {
        println!(
            "  {:<30} {:>8} {:>10}",
            syscall.name, syscall.calls, syscall.errors
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn benchmark(name: &str, median_ns: u64) -> BenchmarkRecord {
        BenchmarkRecord {
            name: name.to_owned(),
            median_ns,
            alloc_count: median_ns / 10,
            alloc_bytes: median_ns * 2,
            max_alloc_count: median_ns / 20,
            max_alloc_bytes: median_ns,
            sample_count: 10,
            iter_count: 100,
        }
    }

    fn report(records: Vec<BenchmarkRecord>) -> BenchmarkReport {
        BenchmarkReport {
            schema: BASELINE_SCHEMA,
            benchmarks: records,
        }
    }

    #[test]
    fn aggregate_reports_keeps_medians_and_per_run_timings() {
        let records = aggregate_reports(&[
            report(vec![benchmark("stable", 30), benchmark("changing", 100)]),
            report(vec![benchmark("stable", 10), benchmark("changing", 300)]),
            report(vec![benchmark("stable", 20), benchmark("changing", 200)]),
        ]);

        assert_eq!(records[0].name, "changing");
        assert_eq!(records[0].median_ns, 200);
        assert_eq!(records[0].run_median_ns, Some(vec![100, 300, 200]));
        assert_eq!(records[1].name, "stable");
        assert_eq!(records[1].median_ns, 20);
    }

    #[test]
    fn tab_separated_baselines_are_rejected() {
        let error = parse_baseline(
            "# old baseline\nthree\t12\t48\nfour\t12\t3\t48\nsix\t12\t3\t4\t5\t48\n",
            Path::new("baseline.tsv"),
        )
        .unwrap_err();

        assert!(error.contains("invalid baseline JSON"));
    }

    #[test]
    fn schema_one_json_without_instability_data_remains_readable() {
        let json = r#"{
            "schema":1,"host":"test","platform":"linux","arch":"x86_64",
            "mode":"normal","warmup_runs":1,"measured_runs":1,
            "sample_count":null,"sample_size":null,"wall_ns":1,
            "warmup_wall_ns":0,"measured_wall_ns":1,"suite_wall_ns":[1],
            "benchmarks":[{"name":"old","median_ns":2,"alloc_count":3,
            "alloc_bytes":4,"max_alloc_count":5,"max_alloc_bytes":6}]
        }"#;

        let records = parse_baseline(json, Path::new("baseline.json")).unwrap();
        assert_eq!(records[0].name, "old");
        assert_eq!(records[0].run_median_ns, None);
    }

    #[test]
    fn unknown_baseline_schema_is_rejected() {
        let json = r#"{
            "schema":99,"host":"test","platform":"linux","arch":"x86_64",
            "mode":"normal","warmup_runs":0,"measured_runs":1,
            "sample_count":null,"sample_size":null,"wall_ns":1,
            "warmup_wall_ns":0,"measured_wall_ns":1,"suite_wall_ns":[1],
            "benchmarks":[]
        }"#;

        let error = parse_baseline(json, Path::new("baseline.json")).unwrap_err();
        assert!(error.contains("unsupported baseline schema 99"));
    }

    #[test]
    fn comparison_reports_added_removed_and_unstable_benchmarks() {
        let previous = vec![BaselineRecord {
            name: "removed".to_owned(),
            median_ns: 10,
            alloc_count: 1,
            alloc_bytes: 2,
            max_alloc_count: 0,
            max_alloc_bytes: 0,
            run_median_ns: None,
        }];
        let current = vec![BaselineRecord {
            name: "added".to_owned(),
            median_ns: 20,
            alloc_count: 1,
            alloc_bytes: 2,
            max_alloc_count: 0,
            max_alloc_bytes: 0,
            run_median_ns: Some(vec![10, 20]),
        }];

        let output = render_comparison(&previous, &current, "comparison", 120);
        assert!(output.contains("added"));
        assert!(output.contains("removed"));
        assert!(output.contains("10 ns–20 ns"));
    }

    #[test]
    fn narrow_comparison_tables_fit_the_requested_width() {
        let current = vec![BaselineRecord {
            name: "a::benchmark::with::a::long::name".to_owned(),
            median_ns: 20,
            alloc_count: 1,
            alloc_bytes: 2,
            max_alloc_count: 0,
            max_alloc_bytes: 0,
            run_median_ns: Some(vec![10, 20]),
        }];

        let output = render_comparison(&[], &current, "comparison", 60);
        for line in output.lines().skip(1) {
            assert!(line.chars().count() <= 60, "line is too wide: {line:?}");
        }

        let output = render_comparison(&[], &current, "comparison", 48);
        for line in output.lines().skip(1) {
            assert!(line.chars().count() <= 48, "line is too wide: {line:?}");
        }
    }

    #[test]
    fn spread_format_distinguishes_stable_and_unstable_runs() {
        assert_eq!(format_spread(&[10, 10]), "stable");
        assert_eq!(format_spread(&[30, 10, 20]), "10 ns–30 ns");
        assert_eq!(format_spread(&[]), "stable");
    }

    #[test]
    fn syscall_trace_counts_only_calls_inside_markers() {
        let trace = concat!(
            "1 read(0, 0, 1) = 1\n",
            "1 prctl(PR_SET_NAME, \"BENCH_BEGIN\") = 0\n",
            "1 write(1, 0, 1) = 1\n",
            "1 openat(AT_FDCWD, \"missing\", 0) = -1 ENOENT (No such file)\n",
            "1 write(1, 0, 1) = 2\n",
            "1 prctl(PR_SET_NAME, \"BENCH_END\") = 0\n",
            "1 close(1) = 0\n",
        );

        let parsed = parse_syscalls(trace);

        assert_eq!(parsed.marker_status, "bounded");
        assert_eq!(parsed.calls, 3);
        assert_eq!(parsed.errors, 1);
        assert_eq!(
            parsed.syscalls,
            vec![
                SyscallCount {
                    name: "openat".to_owned(),
                    calls: 1,
                    errors: 1,
                },
                SyscallCount {
                    name: "write".to_owned(),
                    calls: 2,
                    errors: 0,
                },
            ]
        );
    }

    #[test]
    fn syscall_json_is_stable_and_marks_diagnostics_as_not_timing() {
        let report = SyscallReport {
            schema: SYSCALL_SCHEMA,
            diagnostic: true,
            timing: false,
            benchmarks: vec![SyscallBenchmark {
                name: "group::bench".to_owned(),
                marker_status: "bounded".to_owned(),
                calls: 3,
                errors: 1,
                syscalls: vec![SyscallCount {
                    name: "openat".to_owned(),
                    calls: 3,
                    errors: 1,
                }],
            }],
        };

        let json = miniserde::json::to_string(&report);

        assert_eq!(
            json,
            r#"{"schema":1,"diagnostic":true,"timing":false,"benchmarks":[{"name":"group::bench","marker_status":"bounded","calls":3,"errors":1,"syscalls":[{"name":"openat","calls":3,"errors":1}]}]}"#
        );
        assert_eq!(miniserde::json::from_str::<SyscallReport>(&json).unwrap(), report);
    }

    #[test]
    fn syscall_trace_reports_incomplete_marker_bounds() {
        let missing_begin = parse_syscalls("1 read(0, 0, 1) = 1\n");
        assert_eq!(missing_begin.marker_status, "missing-begin");
        assert_eq!(missing_begin.calls, 0);

        let missing_end = parse_syscalls(concat!(
            "1 prctl(PR_SET_NAME, \"BENCH_BEGIN\") = 0\n",
            "1 read(0, 0, 1) = 1\n",
        ));
        assert_eq!(missing_end.marker_status, "missing-end");
        assert_eq!(missing_end.calls, 1);
    }
}
