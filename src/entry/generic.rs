use std::{
    any::{Any, TypeId},
    cmp::Ordering,
    mem::ManuallyDrop,
    sync::OnceLock,
};

use crate::{
    entry::{BenchEntryRunner, GroupEntry},
    util::sort::natural_cmp,
};

/// A benchmark entry for one generic instantiation.
pub struct GenericBenchEntry {
    /// The associated benchmark group.
    pub group: &'static GroupEntry,

    /// The entry point for the benchmark function.
    pub bench: BenchEntryRunner,

    /// The instantiated type, if any.
    pub ty: Option<EntryType>,

    /// The instantiated const value, if any.
    pub const_value: Option<EntryConst>,
}

impl GenericBenchEntry {
    pub(crate) fn raw_name(&self) -> &str {
        match (&self.ty, &self.const_value) {
            (_, Some(const_value)) => const_value.name(),
            (Some(ty), None) => ty.raw_name(),
            (None, None) => unreachable!(),
        }
    }

    pub(crate) fn display_name(&self) -> &str {
        match (&self.ty, &self.const_value) {
            (_, Some(const_value)) => const_value.name(),
            (Some(ty), None) => ty.display_name(),
            (None, None) => unreachable!(),
        }
    }

    pub(crate) fn path_components(&self) -> impl Iterator<Item = &str> {
        let module_path = self.group.meta.module_path_components();

        let group_component = self.group.meta.raw_name;

        let type_component = if self.const_value.is_some() {
            self.ty.as_ref().map(|ty| ty.display_name())
        } else {
            None
        };

        module_path
            .chain(Some(group_component))
            .chain(type_component)
    }
}

/// Type identity and names for a generic benchmark.
pub struct EntryType {
    get_type_name: fn() -> &'static str,

    #[allow(dead_code)]
    get_type_id: fn() -> TypeId,
}

impl EntryType {
    /// Creates an entry for `T`.
    pub const fn new<T: Any>() -> Self {
        Self {
            get_type_name: std::any::type_name::<T>,
            get_type_id: TypeId::of::<T>,
        }
    }

    pub(crate) fn raw_name(&self) -> &'static str {
        (self.get_type_name)()
    }

    pub(crate) fn display_name(&self) -> &'static str {
        let mut type_name = self.raw_name();

        while let Some((prev, next)) = type_name.split_once("::") {
            if prev.contains('<') {
                break;
            }
            type_name = next;
        }

        type_name
    }
}

/// A const value used by a generic benchmark.
pub struct EntryConst {
    value: *const (),

    partial_cmp: unsafe fn(*const (), *const ()) -> Option<Ordering>,

    to_string: unsafe fn(*const ()) -> String,

    cached_string: ManuallyDrop<OnceLock<&'static str>>,
}

// SAFETY: `T: Send + Sync`.
unsafe impl Send for EntryConst {}
unsafe impl Sync for EntryConst {}

impl EntryConst {
    /// Creates an entry for `value`.
    pub const fn new<T>(value: &'static T) -> Self
    where
        T: PartialOrd + ToString + Send + Sync,
    {
        unsafe fn partial_cmp<T: PartialOrd>(a: *const (), b: *const ()) -> Option<Ordering> {
            unsafe { T::partial_cmp(&*a.cast(), &*b.cast()) }
        }

        unsafe fn to_string<T: ToString>(value: *const ()) -> String {
            unsafe { T::to_string(&*value.cast()) }
        }

        Self {
            value: value as *const T as *const (),
            partial_cmp: partial_cmp::<T>,
            to_string: to_string::<T>,
            cached_string: ManuallyDrop::new(OnceLock::new()),
        }
    }

    pub(crate) fn cmp_name(&self, other: &Self) -> Ordering {
        // SAFETY: If both constants have the same `partial_cmp` function
        // pointer, they are safely comparable. In the context of how this
        // method is used, it is because the constants are of the same type.
        //
        // We don't need a type ID check because constants that are compared to
        // each other all come from the same code generation unit, so their
        // `partial_cmp` function pointers will never differ.
        #[allow(unpredictable_function_pointer_comparisons)]
        if self.partial_cmp == other.partial_cmp
            && let Some(ordering) = unsafe { (self.partial_cmp)(self.value, other.value) }
            && !ordering.is_eq()
        {
            return ordering;
        }

        natural_cmp(self.name(), other.name())
    }

    #[inline]
    pub(crate) fn name(&self) -> &str {
        self.cached_string.get_or_init(|| {
            // SAFETY: The function is guaranteed to call `T::to_string`.
            let string = unsafe { (self.to_string)(self.value) };

            Box::leak(string.into_boxed_str())
        })
    }
}
