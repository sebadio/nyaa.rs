use iced::{Color, Theme, border::Radius, widget::container};

use crate::appearance::tokens;

pub fn sidebar(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(theme.palette().background.weakest.color.into()),
        ..Default::default()
    }
}

pub fn status_bar(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(theme.palette().background.weakest.color.into()),
        ..Default::default()
    }
}

pub fn modal(theme: &Theme) -> container::Style {
    let palette = theme.palette();

    container::Style {
        border: iced::Border {
            color: palette.background.strong.color,
            width: tokens::BORDER_THICK,
            radius: Radius::from(tokens::RADIUS_MEDIUM),
        },
        background: Some(palette.background.base.color.into()),
        ..Default::default()
    }
}

pub fn modal_backdrop(theme: &Theme) -> container::Style {
    let black = theme.palette().background.weakest.color;

    container::Style {
        background: Some(Color { a: 0.8, ..black }.into()),
        ..container::Style::default()
    }
}
