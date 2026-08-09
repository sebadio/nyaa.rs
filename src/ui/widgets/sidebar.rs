use crate::nyaa_app::{NyaaMessage, ScreenKind};
use iced::Element;
use iced::Length::Fill;
use iced::widget::{button, column, row, space, text};
use iced_fonts::lucide::settings;

pub(crate) fn sidebar() -> Element<'static, NyaaMessage> {
    column![
        row![text("Nyaa.rs").size(36)],
        space().height(20),
        button(text("Nyaa Search"))
            .on_press(NyaaMessage::Navigate(ScreenKind::Search))
            .width(Fill),
        button(text("Library"))
            .on_press(NyaaMessage::Navigate(ScreenKind::Library))
            .width(Fill),
        space().height(Fill),
        button(settings())
            .on_press(NyaaMessage::Navigate(ScreenKind::Settings))
            .width(Fill)
    ]
    .spacing(8)
    .width(200)
    .into()
}
