//! Representative workloads for exercising Rustybench as a Cargo benchmark
//! consumer.
//!
//! These are deliberately large enough to measure useful work rather than the
//! cost of an individual arithmetic operation. The target also exercises the
//! public registration, argument, input, counter, const-generic, and threaded
//! benchmark APIs through the real `rustybench::main` entry point.

use std::{collections::HashMap, fmt::Write as _};

use rustybench::{
    Bencher, black_box,
    counter::{BytesCount, ItemsCount},
};

const SIZES: &[usize] = &[256, 4096];
const THREADS: &[usize] = &[1, 2, 4];
const PROBE_SIZES: &[usize] = &[64, 256, 1024, 4096];

#[derive(Clone)]
struct Record {
    key: u64,
    payload: String,
}

#[derive(Clone, Copy)]
struct Event {
    bucket: usize,
    value: u64,
}

fn make_records(size: usize) -> Vec<Record> {
    (0..size)
        .rev()
        .map(|index| Record {
            key: (index as u64 * 1_103_515_245 + 12_345) % size.max(1) as u64,
            payload: format!("record-{index:08}"),
        })
        .collect()
}

fn make_document(size: usize) -> String {
    let mut document = String::with_capacity(size * 24);
    for index in 0..size {
        writeln!(document, "event-{index:08}|{}", index * 17 + 3).unwrap();
    }
    document
}

fn make_events(size: usize) -> Vec<Event> {
    (0..size)
        .map(|index| Event {
            bucket: (index * 17) % 32,
            value: (index as u64).wrapping_mul(31).wrapping_add(7),
        })
        .collect()
}

#[rustybench::bench_group]
mod workloads {
    use super::*;

    /// Sorts records with non-trivial payloads while input generation and
    /// destruction remain outside the timed operation.
    #[rustybench::bench(args = SIZES)]
    fn sort_records(bencher: Bencher, size: usize) {
        bencher
            .counter(ItemsCount::new(size))
            .with_inputs(move || make_records(size))
            .bench_local_refs(|records| {
                records.sort_unstable_by_key(|record| record.key);
                black_box(records.last().map(|record| record.payload.len()))
            });
    }

    /// Parses a moderately sized line-oriented document and aggregates its
    /// values, including a byte-throughput counter derived from each input.
    #[rustybench::bench(args = SIZES)]
    fn parse_document(bencher: Bencher, size: usize) {
        bencher
            .with_inputs(move || make_document(size))
            .input_counter(|document: &String| BytesCount::of_str(document))
            .bench_local_values(|document| {
                let total = document
                    .lines()
                    .map(|line| {
                        let (name, value) = line.split_once('|').unwrap();
                        name.len() + value.parse::<usize>().unwrap()
                    })
                    .sum::<usize>();
                black_box(total)
            });
    }

    /// Aggregates a substantial event batch while exercising the runner's
    /// multi-threaded input path.
    #[rustybench::bench(threads = THREADS)]
    fn aggregate_events(bencher: Bencher) {
        bencher
            .with_inputs(|| make_events(16_384))
            .input_counter(|events: &Vec<Event>| ItemsCount::new(events.len()))
            .bench_values(|events| {
                let mut totals = [0u64; 32];
                for event in events {
                    totals[event.bucket] = totals[event.bucket].wrapping_add(event.value);
                }
                black_box(totals)
            });
    }

    /// Builds a frequency table over a realistic input size, exercising const
    /// expansion through the same executable that runs the other benchmarks.
    #[rustybench::bench(consts = [1024, 8192])]
    fn histogram<const N: usize>(bencher: Bencher) {
        bencher.counter(ItemsCount::new(N)).bench_local(|| {
            let mut counts = HashMap::<u8, usize>::with_capacity(64);
            for index in 0..N {
                let value = (index as u8).wrapping_mul(29);
                *counts.entry(value % 64).or_default() += 1;
            }
            black_box(counts)
        });
    }
}

/// Deliberately tiny, opt-in diagnostics for separating runner overhead from
/// application work. These are ignored in ordinary suites because they are
/// controls, not useful application benchmarks.
#[rustybench::bench_group(ignore)]
mod harness_overhead {
    use super::*;

    #[rustybench::bench]
    fn empty() {
        black_box(1usize);
    }

    #[rustybench::bench]
    fn fixed_work(bencher: Bencher) {
        bencher.with_inputs(|| 256usize).bench_local_values(|size| {
            let mut total = 0usize;
            for index in 0..size {
                total = total.wrapping_add(black_box(index));
            }
            black_box(total)
        });
    }

    #[rustybench::bench(args = PROBE_SIZES)]
    fn fixed_work_by_size(bencher: Bencher, size: usize) {
        bencher.bench_local(|| {
            let mut total = 0usize;
            for index in 0..size {
                total = total.wrapping_add(black_box(index));
            }
            black_box(total)
        });
    }

    #[rustybench::bench(threads = THREADS)]
    fn fixed_threaded_work(bencher: Bencher) {
        bencher.bench(|| {
            let mut total = 0usize;
            for index in 0..256 {
                total = total.wrapping_add(black_box(index));
            }
            black_box(total)
        });
    }
}

fn main() {
    rustybench::main();
}
