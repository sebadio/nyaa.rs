use iced::{Theme, widget::text};

pub fn seeder(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().success.base.color),
    }
}

pub fn leecher(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().danger.base.color),
    }
}

pub fn valid(theme: &Theme) -> text::Style {
    text::Style {
        color: theme.palette().success.weak.color.into(),
    }
}

pub fn invalid(theme: &Theme) -> text::Style {
    text::Style {
        color: theme.palette().danger.weak.color.into(),
    }
}
