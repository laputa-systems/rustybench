use std::{cmp::Ordering, ptr::NonNull};

use crate::{
    benchmark::{BenchOptions, DEFAULT_SAMPLE_COUNT},
    config::SortingAttr,
    counter::KnownCounterKind,
    entry::{AnyBenchEntry, EntryLocation, EntryMeta, GenericBenchEntry, GroupEntry},
    tree_painter::TreeColumn,
    util::sort::natural_cmp,
};

use super::generic::{PathComponent, PathIdentity};

pub(crate) enum EntryTree<'a> {
    Parent {
        identity: PathIdentity,
        raw_name: &'a str,
        display_name: Option<&'a str>,
        group: Option<&'a GroupEntry>,
        children: Vec<Self>,
    },

    Leaf {
        entry: AnyBenchEntry<'a>,

        args: Option<Vec<&'static &'static str>>,

        display_name: Option<&'a str>,
    },
}

impl<'a> EntryTree<'a> {
    pub fn from_benches<I>(benches: I) -> Vec<Self>
    where
        I: IntoIterator<Item = AnyBenchEntry<'a>>,
    {
        let mut result = Vec::<Self>::new();

        for bench in benches {
            match bench {
                AnyBenchEntry::Bench(entry) => {
                    let components = entry.meta.module_path_components().map(PathComponent::name);
                    Self::insert_entry(&mut result, bench, components);
                }
                AnyBenchEntry::GenericBench(entry) => {
                    Self::insert_entry(&mut result, bench, entry.path_components());
                }
            }
        }

        Self::disambiguate(&mut result);

        result
    }

    pub fn max_name_span(tree: &[Self], depth: usize) -> usize {
        const DEPTH_COLS: usize = 3;

        tree.iter()
            .map(|node| {
                let node_name_span = {
                    let prefix_len = depth * DEPTH_COLS;
                    let name_len = node.display_name().chars().count();
                    prefix_len + name_len
                };

                let children_max_span = Self::max_name_span(node.children(), depth + 1);

                let args_max_span = node
                    .arg_names()
                    .unwrap_or_default()
                    .iter()
                    .map(|arg| {
                        let prefix_len = (depth + 1) * DEPTH_COLS;
                        let name_len = arg.chars().count();
                        prefix_len + name_len
                    })
                    .max()
                    .unwrap_or_default();

                node_name_span.max(children_max_span).max(args_max_span)
            })
            .max()
            .unwrap_or_default()
    }

    pub fn common_column_width(tree: &[Self], column: TreeColumn) -> usize {
        if column.is_time_stat() {
            return KnownCounterKind::MAX_COMMON_COLUMN_WIDTH;
        }

        tree.iter()
            .map(|tree| {
                let Some(options) = tree.bench_options() else {
                    return 0;
                };

                let width = match column {
                    TreeColumn::Samples => {
                        let sample_count = options.sample_count.unwrap_or(DEFAULT_SAMPLE_COUNT);
                        1 + sample_count.checked_ilog10().unwrap_or_default() as usize
                    }

                    _ => 0,
                };

                width.max(Self::common_column_width(tree.children(), column))
            })
            .max()
            .unwrap_or_default()
    }

    pub fn insert_group(mut tree: &mut [Self], group: &'a GroupEntry) {
        'component: for component in group.meta.module_path_components() {
            for subtree in tree {
                match subtree {
                    EntryTree::Parent {
                        identity: PathIdentity::Name(name),
                        children,
                        ..
                    } if component == *name => {
                        tree = children;
                        continue 'component;
                    }
                    _ => {}
                }
            }

            return;
        }

        let identity = PathIdentity::Name(group.meta.raw_name);

        for subtree in tree {
            match subtree {
                EntryTree::Parent {
                    identity: node_identity,
                    group: slot,
                    ..
                } if *node_identity == identity => {
                    *slot = Some(group);
                    return;
                }
                _ => {}
            }
        }
    }

    pub fn retain(tree: &mut Vec<Self>, mut filter: impl FnMut(&str) -> bool) {
        fn retain(
            tree: &mut Vec<EntryTree>,
            parent_path: &str,
            filter: &mut impl FnMut(&str) -> bool,
        ) {
            tree.retain_mut(|subtree| {
                let subtree_path: String;
                let subtree_path: &str = if parent_path.is_empty() {
                    subtree.display_name()
                } else {
                    subtree_path = format!("{parent_path}::{}", subtree.display_name());
                    &subtree_path
                };

                match subtree {
                    EntryTree::Parent { children, .. } => {
                        retain(children, subtree_path, filter);

                        !children.is_empty()
                    }

                    EntryTree::Leaf { args: None, .. } => filter(subtree_path),

                    EntryTree::Leaf {
                        args: Some(args), ..
                    } => {
                        args.retain(|arg| filter(&format!("{subtree_path}::{arg}")));

                        !args.is_empty()
                    }
                }
            });
        }
        retain(tree, "", &mut filter);
    }

    pub fn sort_by_attr(tree: &mut [Self], attr: SortingAttr, reverse: bool) {
        let apply_reverse = |ordering: Ordering| {
            if reverse {
                ordering.reverse()
            } else {
                ordering
            }
        };

        tree.sort_unstable_by(|a, b| apply_reverse(a.cmp_by_attr(b, attr)));

        tree.iter_mut().for_each(|tree| {
            match tree {
                EntryTree::Leaf { args, .. } => {
                    if let Some(args) = args {
                        args.sort_by(|&a, &b| apply_reverse(attr.cmp_bench_arg_names(a, b)));
                    }
                }

                EntryTree::Parent { children, .. } => {
                    Self::sort_by_attr(children, attr, reverse);
                }
            }
        });
    }

    fn cmp_by_attr(&self, other: &Self, attr: SortingAttr) -> Ordering {
        let entry_addr_ordering = match (self.entry_addr(), other.entry_addr()) {
            (Some(a), Some(b)) => Some(a.cmp(&b)),
            _ => None,
        };

        if matches!(entry_addr_ordering, Some(Ordering::Equal)) {
            return Ordering::Equal;
        }

        for attr in attr.with_tie_breakers() {
            let ordering = match attr {
                SortingAttr::Kind => self.kind().cmp(&other.kind()),
                SortingAttr::Name => self.cmp_display_name(other),
                SortingAttr::Location => {
                    let location_ordering = self.location().cmp(&other.location());

                    if location_ordering.is_eq() {
                        entry_addr_ordering.unwrap_or(Ordering::Equal)
                    } else {
                        location_ordering
                    }
                }
            };

            if ordering.is_ne() {
                return ordering;
            }
        }

        Ordering::Equal
    }

    fn insert_entry(
        tree: &mut Vec<Self>,
        entry: AnyBenchEntry<'a>,
        mut components: impl Iterator<Item = PathComponent>,
    ) {
        let Some(component) = components.next() else {
            tree.push(Self::Leaf {
                entry,
                args: entry.arg_names().map(|args| args.iter().collect()),
                display_name: None,
            });
            return;
        };

        let Some(children) = Self::get_children(tree, component.identity) else {
            tree.push(Self::from_path(entry, component, components));
            return;
        };

        Self::insert_entry(children, entry, components);
    }

    fn from_path(
        entry: AnyBenchEntry<'a>,
        component: PathComponent,
        mut components: impl Iterator<Item = PathComponent>,
    ) -> Self {
        let child = if let Some(next) = components.next() {
            Self::from_path(entry, next, components)
        } else {
            Self::Leaf {
                entry,
                args: entry.arg_names().map(|args| args.iter().collect()),
                display_name: None,
            }
        };
        Self::Parent {
            identity: component.identity,
            raw_name: component.raw_name,
            display_name: component.display_name,
            group: None,
            children: vec![child],
        }
    }

    fn get_children<'t>(
        tree: &'t mut [Self],
        identity: PathIdentity,
    ) -> Option<&'t mut Vec<Self>> {
        tree.iter_mut().find_map(|tree| match tree {
            Self::Parent {
                identity: node_identity,
                children,
                ..
            } if *node_identity == identity => Some(children),
            _ => None,
        })
    }

    pub fn kind(&self) -> i32 {
        match self {
            Self::Leaf { .. } => 0,
            Self::Parent { .. } => 1,
        }
    }

    pub fn entry_addr(&self) -> Option<NonNull<()>> {
        match self {
            Self::Leaf { entry, .. } => Some(entry.entry_addr()),
            Self::Parent { group, .. } => {
                group.map(|entry: &GroupEntry| NonNull::from(entry).cast())
            }
        }
    }

    pub fn meta(&self) -> Option<&'a EntryMeta> {
        match self {
            Self::Parent { group, .. } => Some(&(*group)?.meta),
            Self::Leaf { entry, .. } => Some(entry.meta()),
        }
    }

    pub fn bench_options(&self) -> Option<&'a BenchOptions<'static>> {
        self.meta()?.bench_options()
    }

    pub fn raw_name(&self) -> &'a str {
        match self {
            Self::Parent {
                group: Some(group), ..
            } => group.meta.raw_name,
            Self::Parent { raw_name, .. } => raw_name,
            Self::Leaf { entry, .. } => entry.raw_name(),
        }
    }

    pub fn display_name(&self) -> &'a str {
        match self {
            Self::Parent {
                display_name: Some(display_name),
                ..
            }
            | Self::Leaf {
                display_name: Some(display_name),
                ..
            } => display_name,

            Self::Leaf { entry, .. } => entry.display_name(),

            Self::Parent { .. } => {
                if let Some(common) = self.meta() {
                    common.display_name
                } else {
                    let raw_name = self.raw_name();
                    raw_name.strip_prefix("r#").unwrap_or(raw_name)
                }
            }
        }
    }

    fn disambiguate(tree: &mut [Self]) {
        for node in tree.iter_mut() {
            if let Self::Parent { children, .. } = node {
                Self::disambiguate(children);
            }
        }

        let mut collisions = Vec::new();

        for (index, node) in tree.iter().enumerate() {
            let Some(full_name) = node.type_identity_name() else {
                continue;
            };

            let collides = tree.iter().enumerate().any(|(other_index, other)| {
                other_index != index && other.display_name() == node.display_name()
            });

            if collides {
                collisions.push((index, full_name));
            }
        }

        for (index, full_name) in collisions {
            tree[index].set_display_name(full_name);
        }
    }

    fn type_identity_name(&self) -> Option<&'a str> {
        match self {
            Self::Parent {
                display_name: Some(_),
                raw_name,
                ..
            } => Some(raw_name),

            Self::Leaf {
                entry: AnyBenchEntry::GenericBench(entry),
                display_name: None,
                ..
            } => match (&entry.ty, &entry.const_value) {
                (Some(ty), None) => Some(ty.raw_name()),
                _ => None,
            },

            _ => None,
        }
    }

    fn set_display_name(&mut self, display_name: &'a str) {
        match self {
            Self::Parent {
                display_name: slot, ..
            }
            | Self::Leaf {
                display_name: slot, ..
            } => *slot = Some(display_name),
        }
    }

    fn location(&self) -> Option<&'a EntryLocation> {
        if let Some(common) = self.meta() {
            Some(&common.location)
        } else {
            self.children().iter().flat_map(Self::location).min()
        }
    }

    fn cmp_display_name(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                Self::Leaf {
                    entry:
                        AnyBenchEntry::GenericBench(GenericBenchEntry {
                            const_value: Some(this),
                            ..
                        }),
                    ..
                },
                Self::Leaf {
                    entry:
                        AnyBenchEntry::GenericBench(GenericBenchEntry {
                            const_value: Some(other),
                            ..
                        }),
                    ..
                },
            ) => this.cmp_name(other),

            _ => natural_cmp(self.display_name(), other.display_name()),
        }
    }

    fn children(&self) -> &[Self] {
        match self {
            Self::Leaf { .. } => &[],
            Self::Parent { children, .. } => children,
        }
    }

    fn arg_names(&self) -> Option<&[&'static &'static str]> {
        match self {
            Self::Leaf { args, .. } => args.as_deref(),
            Self::Parent { .. } => None,
        }
    }
}
