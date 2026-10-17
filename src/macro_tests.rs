//! Focused macro/runner coverage kept in the library test binary.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

use crate::{
    __private::{BENCH_ENTRIES, EntryMeta, GROUP_ENTRIES},
    Rustybench,
};

static CHILD1_ITERS: AtomicUsize = AtomicUsize::new(0);
static CHILD2_ITERS: AtomicUsize = AtomicUsize::new(0);
static CHILD3_ITERS: AtomicUsize = AtomicUsize::new(0);

#[crate::bench_group(crate = crate, sample_count = 10, sample_size = 50)]
mod attr_options {
    use super::*;

    #[crate::bench_group(crate = crate, sample_size = 1)]
    mod child1 {
        use super::*;

        #[crate::bench(crate = crate)]
        fn bench() {
            CHILD1_ITERS.fetch_add(1, SeqCst);
        }
    }

    #[crate::bench_group(crate = crate, sample_count = 42)]
    mod child2 {
        use super::*;

        #[crate::bench(crate = crate)]
        fn bench() {
            CHILD2_ITERS.fetch_add(1, SeqCst);
        }
    }

    mod child3 {
        use super::*;

        #[crate::bench(crate = crate, sample_count = 1)]
        fn bench() {
            CHILD3_ITERS.fetch_add(1, SeqCst);
        }
    }
}

static LIST_PROBE_ITERS: AtomicUsize = AtomicUsize::new(0);

#[crate::bench(crate = crate, ignore)]
fn list_probe() {
    LIST_PROBE_ITERS.fetch_add(1, SeqCst);
}

// These entries exist only to exercise registration metadata. Keeping them
// ignored prevents the iteration-count test from benchmarking its fixtures.
#[crate::bench(crate = crate, ignore)]
fn raw_ident() {}

#[crate::bench(crate = crate, name = "raw name ident", ignore)]
fn raw_name_ident() {}

#[crate::bench_group(crate = crate, ignore)]
mod metadata_group {
    #[crate::bench(crate = crate, ignore)]
    fn inner() {}

    #[crate::bench_group(crate = crate, ignore)]
    mod nested_group {}
}

#[crate::bench(
    crate = crate,
    ignore,
    types = [HashMap<u8, u8>, Vec<HashMap<u8, Vec<u8>>>]
)]
fn generic_types_with_nested_commas<T>() -> T
where
    T: Default,
{
    T::default()
}

#[crate::bench(
    crate = crate,
    ignore,
    args = [Ok::<u8, u8>(1)]
)]
fn args_with_nested_comma_in_type(result: Result<u8, u8>) -> u8 {
    result.unwrap_or(0)
}

#[crate::bench(crate = crate, ignore, consts = [1 << 3, 2])]
fn consts_with_shift<const N: usize>() -> usize {
    N
}

#[crate::bench(crate = crate, ignore, types = [HashMap<fn() -> u8, u8>])]
fn generic_type_with_arrow<T>() {
    let _ = std::marker::PhantomData::<T>;
}

mod collision_a {
    pub struct Widget;
}

mod collision_b {
    pub struct Widget;
}

#[crate::bench(
    crate = crate,
    ignore,
    types = [collision_a::Widget, collision_b::Widget],
    consts = [1u8, 2u8]
)]
fn colliding_consts<T, const N: u8>() {}

#[crate::bench(
    crate = crate,
    ignore,
    types = [collision_a::Widget, collision_b::Widget]
)]
fn colliding_types<T>() {}

#[crate::bench_group(crate = crate, ignore)]
mod terse_override_group {
    #[crate::bench(crate = crate, ignore = false)]
    fn included() {}

    #[crate::bench(crate = crate, ignore)]
    fn excluded() {}
}

fn tree_paths() -> Vec<String> {
    use crate::entry::{AnyBenchEntry, EntryTree};

    let generic = crate::entry::GROUP_ENTRIES
        .iter()
        .flat_map(|group| group.generic_benches_iter().map(AnyBenchEntry::GenericBench));
    let benches = crate::entry::BENCH_ENTRIES
        .iter()
        .map(AnyBenchEntry::Bench)
        .chain(generic);

    let tree = EntryTree::from_benches(benches);

    fn collect(tree: &[EntryTree], parent: &str, out: &mut Vec<String>) {
        for node in tree {
            let path = if parent.is_empty() {
                node.display_name().to_owned()
            } else {
                format!("{parent}::{}", node.display_name())
            };

            match node {
                EntryTree::Leaf { .. } => out.push(path),
                EntryTree::Parent { children, .. } => collect(children, &path, out),
            }
        }
    }

    let mut paths = Vec::new();
    collect(&tree, "", &mut paths);
    paths
}

#[test]
fn generic_type_collisions_produce_distinct_paths() {
    let paths = tree_paths();

    let mut colliding: Vec<&String> = paths
        .iter()
        .filter(|path| path.contains("colliding_consts") || path.contains("colliding_types"))
        .collect();
    colliding.sort();

    assert_eq!(
        colliding.len(),
        6,
        "expected six colliding benchmark paths: {colliding:#?}"
    );

    let mut deduped = colliding.clone();
    deduped.dedup();
    assert_eq!(deduped, colliding, "colliding paths are not distinct");

    assert!(
        colliding
            .iter()
            .any(|path| path.contains("collision_a::Widget"))
    );
    assert!(
        colliding
            .iter()
            .any(|path| path.contains("collision_b::Widget"))
    );
}

fn find_bench(raw_name: &str) -> &'static crate::__private::BenchEntry {
    BENCH_ENTRIES
        .iter()
        .find(|entry| entry.meta.raw_name == raw_name)
        .unwrap_or_else(|| panic!("{raw_name} not found"))
}

fn find_group(raw_name: &str) -> &'static crate::__private::GroupEntry {
    GROUP_ENTRIES
        .iter()
        .find(|entry| entry.meta.raw_name == raw_name)
        .unwrap_or_else(|| panic!("{raw_name} not found"))
}

fn ignored(meta: &EntryMeta) -> bool {
    meta.bench_options
        .as_ref()
        .and_then(|options| options.ignore)
        .unwrap_or_default()
}

#[test]
fn attribute_options_inherit_and_override() {
    Rustybench::default().run_benches();

    assert_eq!(CHILD1_ITERS.load(SeqCst), 10);
    assert_eq!(CHILD2_ITERS.load(SeqCst), 2100);
    assert_eq!(CHILD3_ITERS.load(SeqCst), 50);
}

#[test]
fn list_benches_does_not_run_benchmark_bodies() {
    Rustybench::default().run_only_ignored().list_benches();

    assert_eq!(LIST_PROBE_ITERS.load(SeqCst), 0);
}

#[test]
fn generic_types_with_nested_commas_register() {
    let group = find_group("generic_types_with_nested_commas");
    assert_eq!(group.generic_benches_iter().count(), 2);
}

#[test]
fn args_with_nested_comma_in_type_register() {
    let bench = find_bench("args_with_nested_comma_in_type");
    assert!(ignored(&bench.meta));
}

#[test]
fn consts_with_shift_split() {
    let group = find_group("consts_with_shift");
    assert_eq!(group.generic_benches_iter().count(), 2);
}

#[test]
fn generic_type_with_arrow_register() {
    let group = find_group("generic_type_with_arrow");
    assert_eq!(group.generic_benches_iter().count(), 1);
}

#[test]
fn registration_metadata_preserves_names_and_paths() {
    let raw = find_bench("raw_ident");
    assert_eq!(raw.meta.display_name, "raw_ident");
    assert!(raw.meta.module_path.ends_with("::macro_tests"));
    assert!(ignored(&raw.meta));

    let renamed = find_bench("raw_name_ident");
    assert_eq!(renamed.meta.display_name, "raw name ident");

    let inner = find_bench("inner");
    assert!(
        inner
            .meta
            .module_path
            .ends_with("::macro_tests::metadata_group")
    );
    assert!(ignored(&inner.meta));

    let group = find_group("nested_group");
    assert!(group.meta.module_path.ends_with("::metadata_group"));
    assert!(ignored(&group.meta));
}

#[test]
#[ignore]
fn terse_listing_probe() {
    Rustybench::default().run_action(crate::config::Action::ListTerse);
}

#[test]
fn terse_listing_honors_nested_ignore_override() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "macro_tests::terse_listing_probe",
            "--ignored",
            "--nocapture",
        ])
        .output()
        .expect("run terse listing probe");

    assert!(output.status.success(), "{output:?}");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("terse_override_group::included: benchmark"),
        "nested unignored benchmark missing from terse listing:\n{stdout}"
    );
    assert!(
        !stdout.contains("terse_override_group::excluded"),
        "nested ignored benchmark should not appear:\n{stdout}"
    );
}
