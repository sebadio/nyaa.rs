use std::fmt;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NyaaCategory {
    #[default]
    Anime,
    EnglishTranslated,
    NonEnglishTranslated,
    Raw,
}

impl NyaaCategory {
    pub const ALL: &'static [Self] = &[
        Self::Anime,
        Self::EnglishTranslated,
        Self::NonEnglishTranslated,
        Self::Raw,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Anime => "Anime",
            Self::EnglishTranslated => "English-Translated",
            Self::NonEnglishTranslated => "Non-English-Translated",
            Self::Raw => "RAW",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Anime => "1_0",
            Self::EnglishTranslated => "1_2",
            Self::NonEnglishTranslated => "1_3",
            Self::Raw => "1_4",
        }
    }
}

impl fmt::Display for NyaaCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
