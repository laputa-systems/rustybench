use std::{cmp::Ordering, error::Error, str::FromStr, time::Duration};

use crate::util::sort::natural_cmp;

pub mod filter;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ParsedSeconds(pub Duration);

impl FromStr for ParsedSeconds {
    type Err = Box<dyn Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Duration::try_from_secs_f64(f64::from_str(s)?)?))
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) enum Action {
    #[default]
    Bench,

    Test,

    List,

    ListTerse,
}

#[allow(dead_code)]
impl Action {
    #[inline]
    pub fn is_bench(&self) -> bool {
        matches!(self, Self::Bench)
    }

    #[inline]
    pub fn is_test(&self) -> bool {
        matches!(self, Self::Test)
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        matches!(self, Self::List)
    }

    #[inline]
    pub fn is_list_terse(&self) -> bool {
        matches!(self, Self::ListTerse)
    }
}

#[derive(Copy, Clone, Default)]
pub(crate) enum RunIgnored {
    #[default]
    No,

    Yes,

    Only,
}

impl RunIgnored {
    pub fn run_ignored(self) -> bool {
        matches!(self, Self::Yes | Self::Only)
    }

    pub fn run_non_ignored(self) -> bool {
        matches!(self, Self::Yes | Self::No)
    }

    pub fn should_run(self, ignored: bool) -> bool {
        if ignored {
            self.run_ignored()
        } else {
            self.run_non_ignored()
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SortingAttr {
    #[default]
    Kind,

    Name,

    Location,
}

impl SortingAttr {
    pub fn with_tie_breakers(self) -> [Self; 3] {
        use SortingAttr::*;

        match self {
            Kind => [self, Name, Location],
            Name => [self, Location, Kind],
            Location => [self, Kind, Name],
        }
    }

    pub fn cmp_bench_arg_names(self, a: &&str, b: &&str) -> Ordering {
        for attr in self.with_tie_breakers() {
            let ordering = match attr {
                SortingAttr::Kind => Ordering::Equal,

                SortingAttr::Name => 'ordering: {
                    match (a.parse::<u128>(), a.parse::<u128>()) {
                        (Ok(a_u128), Ok(b_u128)) => break 'ordering a_u128.cmp(&b_u128),

                        (Ok(_), Err(_)) => {
                            if b.parse::<i128>().is_ok() {
                                break 'ordering Ordering::Greater;
                            }
                        }

                        (Err(_), Ok(_)) => {
                            if a.parse::<i128>().is_ok() {
                                break 'ordering Ordering::Less;
                            }
                        }

                        (Err(_), Err(_)) => {
                            if let (Ok(a_i128), Ok(b_i128)) = (a.parse::<i128>(), a.parse::<i128>())
                            {
                                break 'ordering a_i128.cmp(&b_i128);
                            }
                        }
                    }

                    if let (Ok(a), Ok(b)) = (a.parse::<f64>(), b.parse::<f64>())
                        && let Some(ordering) = a.partial_cmp(&b)
                    {
                        break 'ordering ordering;
                    }

                    natural_cmp(a, b)
                }

                SortingAttr::Location => {
                    let a: *const &str = a;
                    let b: *const &str = b;
                    a.cmp(&b)
                }
            };

            if ordering != Ordering::Equal {
                return ordering;
            }
        }

        Ordering::Equal
    }
}
