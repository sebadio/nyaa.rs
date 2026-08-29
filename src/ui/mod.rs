pub(crate) mod downloads;
pub(crate) mod search;
pub(crate) mod settings;
pub(crate) mod widgets;

use crate::appearance::tokens;
use crate::nyaa_app::{NyaaAppState, NyaaMessage, NyaaView};
pub(crate) use downloads::Downloads;
use iced::Element;
use iced::Length::Fill;
use iced::widget::container;
pub(crate) use search::Search;

pub(crate) fn main_view(app_state: &NyaaAppState) -> Element<'_, NyaaMessage> {
    let content = match &app_state.current_view {
        NyaaView::NyaaSearch(search) => search::Search::view(search).map(NyaaMessage::Search),
        NyaaView::Downloads(downloads) => {
            downloads::Downloads::view(downloads).map(NyaaMessage::Downloads)
        }
        NyaaView::Settings(settings) => {
            settings::Settings::view(settings).map(NyaaMessage::Settings)
        }
    };

    container(content)
        .padding(tokens::ROOT_PADDING)
        .width(Fill)
        .height(Fill)
        .into()
}
