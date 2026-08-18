pub(crate) mod library;
pub(crate) mod search;
pub(crate) mod settings;
pub(crate) mod widgets;

use crate::nyaa_app::{NyaaAppState, NyaaMessage, NyaaView};
use iced::Element;
use iced::Length::Fill;
use iced::widget::container;
pub(crate) use library::Library;
pub(crate) use search::Search;

pub(crate) fn main_view(app_state: &NyaaAppState) -> Element<'_, NyaaMessage> {
    let content = match &app_state.current_view {
        NyaaView::NyaaSearch(search) => search::Search::view(search).map(NyaaMessage::Search),
        NyaaView::QtorLibrary(library) => library::Library::view(library).map(NyaaMessage::Library),
        NyaaView::Settings(settings) => {
            settings::Settings::view(settings).map(NyaaMessage::Settings)
        }
    };

    container(content)
        .padding(12)
        .width(Fill)
        .height(Fill)
        .into()
}
