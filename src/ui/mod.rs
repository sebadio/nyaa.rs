pub(crate) mod library;
pub(crate) mod search;
pub(crate) mod settings;
pub(crate) mod widgets;

use crate::nyaa_app::{NyaaAppState, NyaaMessage, NyaaView, ScreenKind};
use iced::Element;
use iced::Length::Fill;
use iced::widget::{button, column, container, mouse_area, row, space, text};
use iced_fonts::lucide::settings;
use iced_fonts::lucide::{maximize, minus, x};
pub(crate) use library::Library;
pub(crate) use search::Search;

pub(crate) fn custom_titlebar() -> Element<'static, NyaaMessage> {
    mouse_area(
        container(
            row![
                space().width(Fill),
                button(minus()).on_press(NyaaMessage::Minimize),
                button(maximize()).on_press(NyaaMessage::ToggleWindowMode),
                button(x()).on_press(NyaaMessage::Exit),
            ]
            .width(Fill),
        )
        .height(32),
    )
    .on_press(NyaaMessage::Drag)
    .on_double_click(NyaaMessage::ToggleWindowMode)
    .into()
}

pub(crate) fn main_view(app_state: &NyaaAppState) -> Element<'_, NyaaMessage> {
    match &app_state.current_view {
        NyaaView::NyaaSearch(search) => search::Search::view(search).map(NyaaMessage::Search),
        NyaaView::QtorLibrary(library) => library::Library::view(library).map(NyaaMessage::Library),
        NyaaView::Settings(settings) => {
            settings::Settings::view(settings).map(NyaaMessage::Settings)
        }
    }
}

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
