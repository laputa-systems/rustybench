use regex_lite::Regex;

use crate::util::split_vec::SplitVec;

pub(crate) enum Filter {
    Regex(Regex),
    Exact(String),
}

impl Filter {
    fn is_match(&self, s: &str) -> bool {
        match self {
            Self::Regex(regex) => regex.is_match(s),
            Self::Exact(exact) => exact == s,
        }
    }
}

#[derive(Default)]
pub(crate) struct FilterSet {
    filters: SplitVec<Filter>,
}

impl FilterSet {
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.filters.reserve_exact(additional);
    }

    #[inline]
    pub fn include(&mut self, filter: Filter) {
        self.insert_filter(filter, true);
    }

    #[inline]
    pub fn exclude(&mut self, filter: Filter) {
        self.insert_filter(filter, false);
    }

    fn insert_filter(&mut self, filter: Filter, inclusive: bool) {
        self.filters.insert(filter, inclusive);
    }

    pub fn is_match(&self, entry_path: &str) -> bool {
        let filters = self.filters.all();
        let inclusive_start = self.filters.split_index();

        if let Some(index) = filters.iter().position(|f| f.is_match(entry_path)) {
            return index >= inclusive_start;
        }

        filters.len() == inclusive_start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let filters = FilterSet::default();
        assert!(filters.is_match("abc"));
        assert!(filters.is_match("123"));
    }

    mod single {
        use super::*;

        #[test]
        fn inclusive_exact() {
            let mut filters = FilterSet::default();

            filters.include(Filter::Exact("abc".into()));

            assert!(filters.is_match("abc"));
            assert!(!filters.is_match("ab"));
            assert!(!filters.is_match("abcd"));
        }

        #[test]
        fn exclusive_exact() {
            let mut filters = FilterSet::default();

            filters.exclude(Filter::Exact("abc".into()));

            assert!(!filters.is_match("abc"));
            assert!(filters.is_match("ab"));
            assert!(filters.is_match("abcd"));
        }

        #[test]
        fn inclusive_regex() {
            let mut filters = FilterSet::default();
            let regex = Regex::new("abc.*123").unwrap();

            filters.include(Filter::Regex(regex));

            assert!(!filters.is_match("abc"));
            assert!(filters.is_match("abc123"));
            assert!(filters.is_match("abc::123"));
        }

        #[test]
        fn exclusive_regex() {
            let mut filters = FilterSet::default();
            let regex = Regex::new("abc.*123").unwrap();

            filters.exclude(Filter::Regex(regex));

            assert!(filters.is_match("abc"));
            assert!(!filters.is_match("abc123"));
            assert!(!filters.is_match("abc::123"));
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn exact() {
            let mut filters = FilterSet::default();

            filters.include(Filter::Exact("abc".into()));
            filters.include(Filter::Exact("123".into()));

            assert!(filters.is_match("abc"));
            assert!(filters.is_match("123"));
            assert!(!filters.is_match("xyz"));
        }
    }

    mod overridden {
        use super::*;

        #[test]
        fn exact() {
            let mut filters = FilterSet::default();

            filters.include(Filter::Exact("abc".into()));
            filters.exclude(Filter::Exact("abc".into()));

            assert!(!filters.is_match("abc"));
        }

        #[test]
        fn regex() {
            let mut filters = FilterSet::default();
            let regex = Regex::new("abc.*123").unwrap();

            filters.include(Filter::Regex(regex.clone()));
            filters.exclude(Filter::Regex(regex));

            assert!(!filters.is_match("abc::123"));
            assert!(!filters.is_match("123::abc"));
        }
    }
}
