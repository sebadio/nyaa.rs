mod nyaa_app;
mod qbittorrent;
mod ui;
use iced::window::Settings;
use iced::{Font, Size};
use iced_fonts::LUCIDE_FONT_BYTES;
use nyaa_app::NyaaAppState;

fn main() -> iced::Result {
    env_logger::init();

    let original_size = Size {
        width: 1280.0,
        height: 700.0,
    };

    iced::application(NyaaAppState::new, NyaaAppState::update, NyaaAppState::view)
        .default_font(Font::with_name("Monocraft"))
        // .default_font(Font::with_name("Miracode"))
        // .decorations(false)
        .font(LUCIDE_FONT_BYTES)
        .theme(iced::Theme::Ferra)
        .window(Settings {
            min_size: Some(original_size),
            size: original_size,
            ..Settings::default()
        })
        .title("Nyaa.rs")
        .subscription(NyaaAppState::subscription)
        .run()
}
