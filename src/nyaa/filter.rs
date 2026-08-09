use std::fmt;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NyaaFilter {
    #[default]
    NoFilter,
    NoRemakes,
    TrustedOnly,
}

impl NyaaFilter {
    pub const ALL: &'static [Self] = &[Self::NoFilter, Self::NoRemakes, Self::TrustedOnly];

    pub fn name(&self) -> &'static str {
        match self {
            Self::NoFilter => "No Filter",
            Self::NoRemakes => "No Remakes",
            NyaaFilter::TrustedOnly => "Trusted Only",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NoFilter => "0",
            NyaaFilter::NoRemakes => "1",
            NyaaFilter::TrustedOnly => "2",
        }
    }
}

impl fmt::Display for NyaaFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
