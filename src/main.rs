pub mod config;
pub mod nyaa_app;
pub mod nyaa_search;
pub mod qbittorrent;
pub mod ui;

use crate::config::Config;
use iced::window::Settings;
use iced::{Font, Size};
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
    .default_font(Font::with_name("Monocraft"))
    .decorations(decorations)
    .font(LUCIDE_FONT_BYTES)
    .theme(NyaaAppState::theme)
    .window(Settings {
        min_size: Some(window_size),
        size: window_size,
        ..Settings::default()
    })
    .title("Nyaa.rs")
    .subscription(NyaaAppState::subscription)
    .run()
}
