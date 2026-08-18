pub mod config;
pub mod nyaa_app;
pub mod ui;
pub mod util;

use crate::config::Config;
use iced::window::Settings;
use iced::{Font, Pixels, Size};
use iced_fonts::LUCIDE_FONT_BYTES;
use nyaa_app::NyaaAppState;

fn main() -> iced::Result {
    env_logger::init();

    let config = Config::load().unwrap_or_default();
    let decorations = !config.uses_custom_titlebar;
    let window_size = Size {
        height: config.window_size.height,
        width: config.window_size.width,
    };

    iced::application(
        move || NyaaAppState::new(config.clone()),
        NyaaAppState::update,
        NyaaAppState::view,
    )
    .decorations(decorations)
    .theme(NyaaAppState::theme)
    .window(Settings {
        min_size: Some(window_size),
        size: window_size,
        ..Settings::default()
    })
    .settings(iced::Settings {
        default_text_size: Pixels(14.0),
        default_font: Font::new("Monocraft"), // FIX <- This is slowing down the startup
        fonts: vec![LUCIDE_FONT_BYTES.into()],
        ..Default::default()
    })
    .title("Nyaa.rs")
    .subscription(NyaaAppState::subscription)
    .run()
}
