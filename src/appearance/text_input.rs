use iced::border::Radius;
use iced::widget::text_input;
use iced::{Border, Theme};

use crate::appearance::tokens;

pub fn primary(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.palette();

    let border_color = match status {
        text_input::Status::Active => palette.background.weak.color,
        text_input::Status::Hovered | text_input::Status::Focused { .. } => {
            palette.background.strongest.color
        }
        text_input::Status::Disabled => palette.background.base.color,
    };

    text_input::Style {
        border: Border {
            radius: Radius::from(tokens::RADIUS_LARGE),
            width: tokens::BORDER_THICK,
            color: border_color,
            ..Default::default()
        },
        ..text_input::default(theme, status)
    }
}
