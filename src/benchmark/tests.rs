
use std::{
    collections::HashSet,
    sync::atomic::{AtomicUsize, Ordering::SeqCst},
};

use super::*;
use crate::{config::Action, util::thread::ThreadPool};

const SAMPLE_COUNT: u32 = 3;

const SAMPLE_SIZE: u32 = 2;

const THREAD_COUNTS: &[usize] = &[1, 2, 3, 4, 5, 6, 9];

#[track_caller]
fn test_bencher(test: &mut dyn FnMut(Bencher)) {
    let bench_options = BenchOptions {
        sample_count: Some(SAMPLE_COUNT),
        sample_size: Some(SAMPLE_SIZE),
        ..BenchOptions::default()
    };

    for action in [Action::Bench, Action::Test] {
        let shared_context = SharedContext {
            action,
            thread_pool: ThreadPool::new(),
        };

        for &thread_count in THREAD_COUNTS {
            let mut bench_context = BenchContext::new(
                &shared_context,
                &bench_options,
                NonZeroUsize::new(thread_count).unwrap(),
            );

            test(Bencher::new(&mut bench_context));

            assert!(bench_context.did_run);

            let samples = &bench_context.samples;

            if action.is_test() {
                assert_eq!(samples.time_samples.capacity(), 0);
            }
        }
    }
}

fn make_string() -> String {
    ('a'..='z').collect()
}

#[allow(clippy::unused_unit)]
mod run_count {
    use super::*;

    fn test(run_bench: fn(Bencher, &(dyn Fn() + Sync))) {
        test_with_drop_counter(&AtomicUsize::new(usize::MAX), run_bench);
    }

    fn test_with_drop_counter(
        drop_count: &AtomicUsize,
        run_bench: fn(Bencher, &(dyn Fn() + Sync)),
    ) {
        let test_drop_count = drop_count.load(SeqCst) != usize::MAX;

        let bench_count = AtomicUsize::new(0);
        let test_count = AtomicUsize::new(0);

        let mut thread_counts = HashSet::<u32>::new();
        test_bencher(&mut |bencher| {
            let context = &bencher.context;

            let thread_count = context.thread_count.get();
            thread_counts.insert(thread_count as u32);

            let is_test = context.shared_context.action.is_test();

            let shared_run_count = if is_test { &test_count } else { &bench_count };
            let start_run_count = shared_run_count.load(SeqCst);

            run_bench(bencher, &|| {
                shared_run_count.fetch_add(1, SeqCst);
            });

            let end_run_count = shared_run_count.load(SeqCst);
            let run_count = end_run_count - start_run_count;

            if is_test {
                assert_eq!(run_count, thread_count);
            } else {
                let expected_samples = match SAMPLE_COUNT as usize % thread_count {
                    0 => SAMPLE_COUNT,
                    rem => SAMPLE_COUNT + (thread_count - rem) as u32,
                };

                let expected_iters = (expected_samples * SAMPLE_SIZE) as usize;
                assert_eq!(run_count, expected_iters);
            }
        });

        let thread_count = thread_counts.into_iter().sum::<u32>();

        let bench_count = bench_count.into_inner() as u32;
        let test_count = test_count.into_inner() as u32;

        let total_count = bench_count + test_count;
        assert_ne!(total_count, 0);

        if test_drop_count {
            assert_eq!(drop_count.load(SeqCst), total_count as usize);
        }

        assert_eq!(test_count, thread_count);
    }

    #[test]
    fn bench() {
        struct DroppedZst;

        static ZST_DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        impl Drop for DroppedZst {
            fn drop(&mut self) {
                ZST_DROP_COUNT.fetch_add(1, SeqCst);
            }
        }

        test(|b, f| b.bench(f));

        test(|b, f| {
            b.bench(|| -> i32 {
                f();
                100i32
            })
        });

        test(|b, f| {
            b.bench(|| -> String {
                f();
                make_string()
            })
        });

        test_with_drop_counter(&ZST_DROP_COUNT, |b, f| {
            b.bench(|| -> DroppedZst {
                f();
                DroppedZst
            })
        });
    }

    #[test]
    fn bench_values() {
        struct DroppedZst;

        static ZST_DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        impl Drop for DroppedZst {
            fn drop(&mut self) {
                ZST_DROP_COUNT.fetch_add(1, SeqCst);
            }
        }

        let test_zst_drop = |run_bench| {
            ZST_DROP_COUNT.store(0, SeqCst);
            test_with_drop_counter(&ZST_DROP_COUNT, run_bench);
        };

        test(|b, f| b.with_inputs(|| ()).bench_values(|_: ()| -> () { f() }));

        test(|b, f| {
            b.with_inputs(|| ()).bench_values(|_: ()| -> i32 {
                f();
                100i32
            })
        });

        test(|b, f| {
            b.with_inputs(|| ()).bench_values(|_: ()| -> String {
                f();
                make_string()
            })
        });

        test_zst_drop(|b, f| {
            b.with_inputs(|| ()).bench_values(|_: ()| -> DroppedZst {
                f();
                DroppedZst
            })
        });

        test(|b, f| {
            b.with_inputs(|| 100i32)
                .bench_values(|_: i32| -> () { f() })
        });

        test(|b, f| {
            b.with_inputs(|| 100i32).bench_values(|value: i32| -> i32 {
                f();
                value
            })
        });

        test(|b, f| {
            b.with_inputs(|| 100i32).bench_values(|_: i32| -> String {
                f();
                make_string()
            })
        });

        test_zst_drop(|b, f| {
            b.with_inputs(|| 100i32)
                .bench_values(|_: i32| -> DroppedZst {
                    f();
                    DroppedZst
                })
        });

        test(|b, f| {
            b.with_inputs(make_string)
                .bench_values(|_: String| -> () { f() })
        });

        test(|b, f| {
            b.with_inputs(make_string).bench_values(|_: String| -> i32 {
                f();
                100i32
            })
        });

        test(|b, f| {
            b.with_inputs(make_string)
                .bench_values(|value: String| -> String {
                    f();
                    value
                })
        });

        test_zst_drop(|b, f| {
            b.with_inputs(make_string)
                .bench_values(|_: String| -> DroppedZst {
                    f();
                    DroppedZst
                })
        });

        test_zst_drop(|b, f| {
            b.with_inputs(|| DroppedZst)
                .bench_values(|_: DroppedZst| -> () { f() })
        });

        test_zst_drop(|b, f| {
            b.with_inputs(|| DroppedZst)
                .bench_values(|_: DroppedZst| -> i32 {
                    f();
                    100i32
                })
        });

        test_zst_drop(|b, f| {
            b.with_inputs(|| DroppedZst)
                .bench_values(|_: DroppedZst| -> String {
                    f();
                    make_string()
                })
        });

        test_zst_drop(|b, f| {
            b.with_inputs(|| DroppedZst)
                .bench_values(|value: DroppedZst| -> DroppedZst {
                    f();
                    value
                })
        });
    }

    #[test]
    fn bench_refs() {
        struct DroppedZst;

        static ZST_DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        impl Drop for DroppedZst {
            fn drop(&mut self) {
                ZST_DROP_COUNT.fetch_add(1, SeqCst);
            }
        }

        let test_zst_drop = |run_bench| {
            ZST_DROP_COUNT.store(0, SeqCst);
            test_with_drop_counter(&ZST_DROP_COUNT, run_bench);
        };

        test(|b, f| b.with_inputs(|| ()).bench_refs(|_: &mut ()| -> () { f() }));

        test(|b, f| {
            b.with_inputs(|| ()).bench_refs(|_: &mut ()| -> i32 {
                f();
                100i32
            })
        });

        test(|b, f| {
            b.with_inputs(|| ()).bench_refs(|_: &mut ()| -> String {
                f();
                make_string()
            })
        });

        test_zst_drop(|b, f| {
            b.with_inputs(|| ()).bench_refs(|_: &mut ()| -> DroppedZst {
                f();
                DroppedZst
            })
        });

        test(|b, f| {
            b.with_inputs(|| 100i32)
                .bench_refs(|_: &mut i32| -> () { f() })
        });

        test(|b, f| {
            b.with_inputs(|| 100i32)
                .bench_refs(|value: &mut i32| -> i32 {
                    f();
                    *value
                })
        });

        test(|b, f| {
            b.with_inputs(|| 100i32)
                .bench_refs(|_: &mut i32| -> String {
                    f();
                    make_string()
                })
        });

        test_zst_drop(|b, f| {
            b.with_inputs(|| 100i32)
                .bench_refs(|_: &mut i32| -> DroppedZst {
                    f();
                    DroppedZst
                })
        });

        test(|b, f| {
            b.with_inputs(make_string)
                .bench_refs(|_: &mut String| -> () { f() })
        });

        test(|b, f| {
            b.with_inputs(make_string)
                .bench_refs(|_: &mut String| -> i32 {
                    f();
                    100i32
                })
        });

        test(|b, f| {
            b.with_inputs(make_string)
                .bench_refs(|value: &mut String| -> String {
                    f();
                    value.clone()
                })
        });

        test_zst_drop(|b, f| {
            b.with_inputs(make_string)
                .bench_refs(|_: &mut String| -> DroppedZst {
                    f();
                    DroppedZst
                })
        });

        test_zst_drop(|b, f| {
            b.with_inputs(|| DroppedZst)
                .bench_refs(|_: &mut DroppedZst| -> () { f() })
        });

        test_zst_drop(|b, f| {
            b.with_inputs(|| DroppedZst)
                .bench_refs(|_: &mut DroppedZst| -> i32 {
                    f();
                    100i32
                })
        });

        test_zst_drop(|b, f| {
            b.with_inputs(|| DroppedZst)
                .bench_refs(|_: &mut DroppedZst| -> String {
                    f();
                    make_string()
                })
        });

        test_zst_drop(|b, f| {
            b.with_inputs(|| {
                ZST_DROP_COUNT.fetch_sub(1, SeqCst);

                DroppedZst
            })
            .bench_refs(|_: &mut DroppedZst| -> DroppedZst {
                f();
                DroppedZst
            })
        });
    }
}

#[test]
fn setup_cost_bounds_tuned_sample_size() {
    _ = crate::time::bench_overheads();
    _ = crate::time::timer_precision();

    let bench_options = BenchOptions {
        sample_count: Some(1),
        ..BenchOptions::default()
    };
    let shared_context = SharedContext {
        action: Action::Bench,
        thread_pool: ThreadPool::new(),
    };
    let mut bench_context = BenchContext::new(
        &shared_context,
        &bench_options,
        NonZeroUsize::new(1).unwrap(),
    );

    Bencher::new(&mut bench_context)
        .with_inputs(|| vec![0u8; 4 * 1024 * 1024])
        .bench_refs(|buffer: &mut Vec<u8>| buffer.len());

    assert!(
        bench_context.samples.sample_size <= 8,
        "retained {} generated inputs; setup and drop cost should bound tuning",
        bench_context.samples.sample_size,
    );
}

#[test]
fn worker_panic_releases_peers() {
    use std::{sync::mpsc, time::Duration};

    let (result_tx, result_rx) = mpsc::channel();

    std::thread::Builder::new()
        .name("rustybench-panic-test".into())
        .spawn(move || {
            let bench_options = BenchOptions {
                sample_count: Some(SAMPLE_COUNT),
                sample_size: Some(SAMPLE_SIZE),
                ..BenchOptions::default()
            };

            let shared_context = SharedContext {
                action: Action::Bench,
                thread_pool: ThreadPool::new(),
            };

            let mut bench_context = BenchContext::new(
                &shared_context,
                &bench_options,
                NonZeroUsize::new(2).unwrap(),
            );

            let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                Bencher::new(&mut bench_context).bench(|| {
                    if std::thread::current().name() == Some("rustybench-1") {
                        panic!("intentional benchmark panic");
                    }
                });
            }))
            .is_err();

            _ = result_tx.send(panicked);
        })
        .unwrap();

    match result_rx.recv_timeout(Duration::from_secs(10)) {
        Ok(panicked) => assert!(panicked, "benchmark ended without reporting the worker panic"),
        Err(_) => panic!("benchmark workers stayed blocked; the sample barrier did not cancel"),
    }
}

#[test]
fn alloc_median_follows_allocation_order_not_time_order() {
    let bench_options = BenchOptions::default();
    let shared_context = SharedContext {
        action: Action::Bench,
        thread_pool: ThreadPool::new(),
    };
    let mut bench_context = BenchContext::new(
        &shared_context,
        &bench_options,
        NonZeroUsize::new(1).unwrap(),
    );

    bench_context.samples.sample_size = 1;

    let synthetic: [(u128, u64, u64); 3] = [
        (100, 1000, 1000),
        (200, 5, 5),
        (300, 10, 10),
    ];

    for (index, &(duration_picos, max_count, max_size)) in synthetic.iter().enumerate() {
        bench_context.samples.time_samples.push(TimeSample {
            duration: FineDuration {
                picos: duration_picos,
            },
        });
        bench_context.samples.alloc_info_by_sample.insert(
            index as u32,
            ThreadAllocInfo {
                tallies: AllocOpMap {
                    values: AllocOp::ALL.map(|op| {
                        if op == AllocOp::Alloc {
                            AllocTally {
                                count: max_count,
                                size: max_size,
                            }
                        } else {
                            AllocTally::default()
                        }
                    }),
                },
                max_count: max_count as i64,
                max_size: max_size as i64,
                ..Default::default()
            },
        );
    }

    let stats = bench_context.compute_stats();
    let record = crate::report::BenchmarkRecord::from_stats("synthetic", &stats);

    assert_eq!(record.alloc_count, 10);
    assert_eq!(record.alloc_bytes, 10);
    assert_eq!(record.max_alloc_count, 10);
    assert_eq!(record.max_alloc_bytes, 10);
}

mod no_input {
    use super::*;

    #[test]
    fn string_output() {
        test_bencher(&mut |b| b.bench(make_string));
    }

    #[test]
    fn no_output() {
        test_bencher(&mut |b| b.bench(|| black_box_drop(make_string())));
    }
}

mod string_input {
    use super::*;

    #[test]
    fn string_output() {
        test_bencher(&mut |b| {
            b.with_inputs(make_string)
                .bench_values(|s| s.to_ascii_uppercase())
        });
    }

    #[test]
    fn no_output() {
        test_bencher(&mut |b| {
            b.with_inputs(make_string)
                .bench_refs(|s| s.make_ascii_uppercase())
        });
    }
}

mod zst_input {
    use super::*;

    #[test]
    fn zst_output() {
        struct DroppedZst;

        static ZST_COUNT: AtomicUsize = AtomicUsize::new(0);

        impl Drop for DroppedZst {
            fn drop(&mut self) {
                ZST_COUNT.fetch_sub(1, SeqCst);
            }
        }

        test_bencher(&mut |b| {
            b.with_inputs(|| {
                ZST_COUNT.fetch_add(1, SeqCst);
                DroppedZst
            })
            .bench_values(black_box);
        });

        assert_eq!(ZST_COUNT.load(SeqCst), 0);
    }

    #[test]
    fn no_output() {
        struct DroppedZst;

        static ZST_COUNT: AtomicUsize = AtomicUsize::new(0);

        impl Drop for DroppedZst {
            fn drop(&mut self) {
                ZST_COUNT.fetch_sub(1, SeqCst);
            }
        }

        test_bencher(&mut |b| {
            b.with_inputs(|| {
                ZST_COUNT.fetch_add(1, SeqCst);
                DroppedZst
            })
            .bench_values(drop);
        });

        assert_eq!(ZST_COUNT.load(SeqCst), 0);
    }
}

#[test]
#[should_panic(expected = "sample size must be greater than zero")]
fn options_with_zero_sample_size_are_rejected() {
    let bench_options = BenchOptions {
        sample_count: Some(1),
        sample_size: Some(0),
        ..BenchOptions::default()
    };
    let shared_context = SharedContext {
        action: Action::Bench,
        thread_pool: ThreadPool::new(),
    };
    let bench_context = BenchContext::new(
        &shared_context,
        &bench_options,
        NonZeroUsize::new(1).unwrap(),
    );

    let _ = bench_context.initial_mode();
}
