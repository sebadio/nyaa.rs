use iced::border::Radius;
use iced::widget::button::{self, Status};
use iced::{Border, Theme};

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

pub fn title_link(theme: &Theme, status: Status) -> button::Style {
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

pub fn nav_button_style(is_selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.palette();

        let (bg_color, text_color) = match (status, is_selected) {
            (button::Status::Hovered | button::Status::Pressed, true) => (
                palette.background.strong.color,
                palette.background.base.text,
            ),
            (button::Status::Hovered, false) => (
                palette.background.neutral.color,
                palette.background.base.text,
            ),
            (_, true) => (palette.background.base.color, palette.background.base.text),
            (_, false) => (
                palette.background.weakest.color,
                palette.background.base.text,
            ),
        };

        button::Style {
            background: Some(iced::Background::Color(bg_color)),
            border: Border {
                radius: Radius::from(tokens::RADIUS_LARGE),
                ..Default::default()
            },
            text_color,
            ..Default::default()
        }
    }
}
