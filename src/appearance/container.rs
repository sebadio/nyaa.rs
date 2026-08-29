use iced::{Border, Color, Theme, border::Radius, widget::container};

use crate::appearance::tokens;

pub fn weakest(theme: &Theme) -> container::Style {
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

pub fn backdrop(theme: &Theme) -> container::Style {
    let black = theme.palette().background.weakest.color;

    container::Style {
        background: Some(Color { a: 0.8, ..black }.into()),
        ..container::Style::default()
    }
}

pub fn tooltip(theme: &Theme) -> container::Style {
    let palette = theme.palette();

    container::Style {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            color: palette.primary.strong.color,
            width: tokens::BORDER_THICK,
            radius: Radius::from(tokens::RADIUS_SMALL),
        },
        ..Default::default()
    }
}

pub fn badge_success(theme: &Theme) -> container::Style {
    let palette = theme.palette();

    let bg_color = palette.background.weak.color;
    let border = Border {
        radius: Radius::from(tokens::RADIUS_LARGE),
        color: palette.success.weak.color,
        width: tokens::BORDER_THIN,
        ..Default::default()
    };

    container::Style {
        background: Some(bg_color.into()),
        border: border,
        ..Default::default()
    }
}

pub(crate) fn base(theme: &Theme) -> container::Style {
    container::Style {
        border: Border {
            color: theme.palette().background.weak.color,
            width: tokens::BORDER_THICK,
            radius: Radius::from(tokens::RADIUS_LARGE),
        },
        ..Default::default()
    }
}
