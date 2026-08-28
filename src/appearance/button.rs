use iced::Theme;
use iced::border::Radius;
use iced::widget::button::{self, Status};

use crate::appearance::tokens;

pub fn secondary(theme: &Theme, status: Status) -> button::Style {
    let palette = theme.palette();

    let bg = match status {
        Status::Active => palette.background.weaker.color,
        Status::Hovered | Status::Pressed => palette.background.strongest.color,
        Status::Disabled => palette.background.stronger.color.scale_alpha(0.5),
    };

    let mut style = button::secondary(theme, status);
    style.background = Some(bg.into());
    style.border.radius = Radius::from(tokens::RADIUS_LARGE);

    style
}

pub fn download(theme: &Theme, status: Status) -> button::Style {
    let palette = theme.palette();

    let bg = match status {
        Status::Active => palette.background.base.color,
        Status::Hovered | Status::Pressed => palette.background.strong.color,
        Status::Disabled => palette.background.weak.color.scale_alpha(0.5),
    };

    let mut style = button::primary(theme, status);
    style.background = Some(bg.into());
    style.border.radius = Radius::from(tokens::RADIUS_SMALL);

    style
}

pub fn title_hover(theme: &Theme, status: Status) -> button::Style {
    let palette = theme.palette();
    let txt_col = match status {
        Status::Hovered => palette.primary.strong.color,
        _ => palette.background.base.text,
    };

    button::Style {
        text_color: txt_col,
        background: None,
        ..Default::default()
    }
}
