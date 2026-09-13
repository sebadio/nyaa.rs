use iced::Theme;
use iced::border::Radius;
use iced::widget::button::{self, Status};

use crate::appearance::tokens;

pub fn secondary(theme: &Theme, status: Status) -> button::Style {
    let palette = theme.palette();

    let (bg, text) = match status {
        Status::Active => (
            palette.background.weaker.color,
            palette.background.weaker.text,
        ),
        Status::Hovered | Status::Pressed => (
            palette.background.strongest.color,
            palette.background.strongest.text,
        ),
        Status::Disabled => (
            palette.background.stronger.color.scale_alpha(0.5),
            palette.background.stronger.text.scale_alpha(0.5),
        ),
    };

    let mut style = button::secondary(theme, status);
    style.background = Some(bg.into());
    style.text_color = text;
    style.border.radius = Radius::from(tokens::RADIUS_LARGE);

    style
}

pub fn secondary_action(theme: &Theme, status: Status) -> button::Style {
    let palette = theme.palette();

    let (bg, text) = match status {
        Status::Active => (
            palette.background.base.color,
            palette.background.strong.text,
        ),
        Status::Hovered | Status::Pressed => (
            palette.background.strong.color,
            palette.background.stronger.text,
        ),
        Status::Disabled => (
            palette.background.weak.color.scale_alpha(0.5),
            palette.background.weak.text.scale_alpha(0.5),
        ),
    };

    let mut style = button::primary(theme, status);
    style.background = Some(bg.into());
    style.text_color = text;
    style.border.radius = Radius::from(tokens::RADIUS_SMALL);

    style
}

pub(crate) fn hover_only(theme: &Theme, status: Status) -> button::Style {
    let palette = theme.palette();

    let mut style = button::text(theme, status);
    style.background = None;
    style.text_color = match status {
        Status::Hovered => palette.primary.strong.color,
        Status::Pressed => palette.danger.strong.color,
        _ => palette.background.base.text,
    };

    style
}

pub fn title_link(theme: &Theme, status: Status) -> button::Style {
    let palette = theme.palette();

    let mut style = button::text(theme, status);
    style.background = None;
    style.text_color = match status {
        Status::Hovered => palette.primary.strong.color,
        _ => palette.background.base.text,
    };

    style
}
