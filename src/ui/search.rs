use iced::Length::Fill;
use iced::alignment::Vertical::Center;
use iced::border::Radius;
use iced::widget::{
    button, center, column, container, pick_list, row, scrollable, space, table, text, text_input,
    tooltip,
};
use iced::{Border, Element, Pixels, Task, Theme, alignment};
use iced_fonts::lucide::{arrow_down, arrow_up, download, search};
use log::{error, info};
use nyaa::filter::NyaaFilter;
use nyaa::request::NyaaRequest;
use nyaa::{NyaaAdapter, NyaaAdapterError, NyaaCategory, NyaaItem};
use thiserror::Error;

use crate::appearance::{self, tokens};

pub(crate) enum Action {
    None,
    Task(Task<NyaaSearchMessage>),
    OpenPostDownload(NyaaItem),
    ShowError(SearchViewError),
}

#[derive(Debug, Clone)]
pub(crate) enum NyaaSearchMessage {
    Search,
    QueryUpdated(String),
    CategoryUpdated(NyaaCategory),
    FilterUpdated(NyaaFilter),
    SearchResults(Result<Vec<NyaaItem>, NyaaAdapterError>),
    DownloadTorrent(NyaaItem),
}

#[derive(Debug, Clone, Error)]
pub(crate) enum SearchViewError {
    #[error("Search failed: {0}")]
    FailedSearch(String),
}

#[derive(Debug)]
pub(crate) struct Search {
    query: String,
    category: NyaaCategory,
    filter: NyaaFilter,
    results: Vec<NyaaItem>,
    has_searched: bool,
    is_loading: bool,
}

impl Search {
    pub(crate) fn new() -> Self {
        Self {
            query: String::new(),
            category: NyaaCategory::default(),
            results: Vec::new(),
            filter: NyaaFilter::default(),
            has_searched: false,
            is_loading: false,
        }
    }

    pub(crate) fn view(&self) -> Element<'_, NyaaSearchMessage> {
        let content: Element<'_, NyaaSearchMessage> =
            match (self.is_loading, self.results.is_empty()) {
                (true, _) => searching_in_progress(),
                (false, false) => search_content(&self.results),
                (false, true) => empty_search_content(self.has_searched),
            };

        column![search_row(&self), content].spacing(12).into()
    }

    pub(crate) fn update(
        &mut self,
        message: NyaaSearchMessage,
        nyaa_client: &NyaaAdapter,
    ) -> Action {
        match message {
            NyaaSearchMessage::QueryUpdated(query) => {
                self.query = query;
                Action::None
            }

            NyaaSearchMessage::Search => {
                info!("Triggered search");
                self.is_loading = true;
                let client = nyaa_client.clone();
                let request = NyaaRequest::new(&self.query)
                    .set_category(self.category)
                    .set_filter(self.filter);

                Action::Task(Task::perform(
                    async move { client.fetch(Some(request)).await },
                    NyaaSearchMessage::SearchResults,
                ))
            }

            NyaaSearchMessage::CategoryUpdated(category) => {
                info!("Changed category to {}", category.name());
                self.category = category;
                Action::None
            }

            NyaaSearchMessage::FilterUpdated(filter) => {
                info!("Changed filter to {}", filter.name());
                self.filter = filter;

                Action::None
            }

            NyaaSearchMessage::SearchResults(Ok(items)) => {
                self.has_searched = true;
                self.is_loading = false;
                self.results = items;

                Action::None
            }

            NyaaSearchMessage::SearchResults(Err(e)) => {
                self.has_searched = true;
                self.is_loading = false;

                error!("Search failed: {e}");
                Action::ShowError(SearchViewError::FailedSearch(format!(
                    "Search has failed! {}",
                    e
                )))
            }

            NyaaSearchMessage::DownloadTorrent(item) => Action::OpenPostDownload(item),
        }
    }
}

fn searching_in_progress() -> Element<'static, NyaaSearchMessage> {
    column![center(text("Loading..."))]
        .width(Fill)
        .height(Fill)
        .into()
}

fn search_content(results: &[NyaaItem]) -> Element<'_, NyaaSearchMessage> {
    let columns = [
        table::column(text("Title"), |item: &NyaaItem| item_title(item)).width(Fill),
        table::column(text("Size"), |item: &NyaaItem| item_size_info(item))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
        table::column(text("Seeders"), |item: &NyaaItem| seeders(item))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
        table::column(text("Leechers"), |item: &NyaaItem| leechers(item))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
        table::column(space(), |item: &NyaaItem| item_download_button(item))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    ];

    container(
        scrollable(table(columns, results))
            .spacing(tokens::SPACING_SMALL)
            .width(Fill)
            .height(Fill),
    )
    .padding(10)
    .style(|theme: &Theme| container::Style {
        border: Border {
            color: theme.palette().background.weak.color,
            width: tokens::BORDER_THICK,
            radius: Radius::from(tokens::RADIUS_LARGE),
        },
        ..Default::default()
    })
    .into()
}

fn empty_search_content(has_searched: bool) -> Element<'static, NyaaSearchMessage> {
    column![center(
        text(if has_searched {
            "No results"
        } else {
            "Let's search something up!"
        })
        .size(24)
    )]
    .width(Fill)
    .height(Fill)
    .into()
}

fn search_row(state: &Search) -> Element<'_, NyaaSearchMessage> {
    let mut txt_inp = text_input("Let's search nyaa!", &state.query)
        .padding(tokens::INPUT_PADDING)
        .line_height(tokens::INPUT_LINE_HEIGHT)
        .size(tokens::INPUT_SIZE)
        .align_x(alignment::Horizontal::Center)
        .width(Fill)
        .style(appearance::text_input::primary);

    let mut btn = button(center(search()))
        .width(tokens::BNT_BASE_SIZE)
        .height(tokens::BNT_BASE_SIZE)
        .padding(tokens::BTN_PADDING)
        .style(appearance::button::secondary);

    if !state.is_loading {
        txt_inp = txt_inp
            .on_submit(NyaaSearchMessage::Search)
            .on_input(NyaaSearchMessage::QueryUpdated);

        btn = btn.on_press(NyaaSearchMessage::Search);
    }

    let input_and_button = row![txt_inp, btn]
        .height(tokens::BNT_BASE_SIZE)
        .align_y(Center)
        .spacing(tokens::SPACING_BASE)
        .width(Fill);

    let filter_and_category = row![
        column![
            text("Filter"),
            pick_list(Some(state.filter), NyaaFilter::ALL, NyaaFilter::to_string)
                .padding(tokens::PICK_LIST_PADDING)
                .width(Fill)
                .on_select(NyaaSearchMessage::FilterUpdated),
        ]
        .spacing(tokens::SPACING_TINY)
        .width(Fill),
        column![
            text("Category"),
            pick_list(
                Some(state.category),
                NyaaCategory::ALL,
                NyaaCategory::to_string
            )
            .padding(tokens::PICK_LIST_PADDING)
            .width(Fill)
            .on_select(NyaaSearchMessage::CategoryUpdated)
        ]
        .spacing(tokens::SPACING_TINY)
        .width(Fill)
    ]
    .align_y(alignment::Vertical::Center)
    .spacing(tokens::SPACING_BASE)
    .width(Fill);

    column![input_and_button, filter_and_category]
        .spacing(tokens::SPACING_SMALL)
        .into()
}

fn item_size_info(item: &NyaaItem) -> Element<'static, NyaaSearchMessage> {
    text(format!("{}", item.size)).into()
}

fn item_download_button(item: &NyaaItem) -> Element<'static, NyaaSearchMessage> {
    container(
        button(download().size(14))
            .on_press(NyaaSearchMessage::DownloadTorrent(item.clone()))
            .style(appearance::button::download),
    )
    .align_y(alignment::Vertical::Center)
    .align_x(alignment::Horizontal::Center)
    .into()
}

fn item_title(item: &NyaaItem) -> Element<'static, NyaaSearchMessage> {
    let trusted: Option<Element<'static, NyaaSearchMessage>> =
        item.trusted.then_some(trusted_badge());

    let title = column![
        button(
            text(item.title.clone())
                .wrapping(text::Wrapping::None)
                .ellipsis(text::Ellipsis::End)
        )
        .padding(0)
        .style(appearance::button::title_link)
        .on_press(NyaaSearchMessage::DownloadTorrent(item.clone()))
        .width(Fill),
        row![trusted, text(format!("{}", item.category))]
            .align_y(Center)
            .spacing(tokens::SPACING_SMALL)
    ]
    .width(Fill);

    tooltip(
        title,
        container(text(item.title.clone()).size(14))
            .width(tokens::TOOLTIP_WIDTH)
            .padding(tokens::TOOLTIP_PADDING)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.palette().background.weak.color.into()),
                border: Border {
                    color: theme.palette().primary.strong.color,
                    width: tokens::BORDER_THICK,
                    radius: Radius::from(4),
                },
                ..Default::default()
            }),
        tooltip::Position::FollowCursor,
    )
    .into()
}

fn seeders(item: &NyaaItem) -> Element<'static, NyaaSearchMessage> {
    let cant = item.seeders;
    let seeder_style = |theme: &Theme| text::Style {
        color: Some(theme.palette().success.base.color),
    };

    row![
        arrow_up().style(seeder_style),
        text(format!("{}", cant)).style(seeder_style)
    ]
    .spacing(4)
    .into()
}

fn leechers(item: &NyaaItem) -> Element<'static, NyaaSearchMessage> {
    let cant = item.leechers;
    let leecher_style = |theme: &Theme| text::Style {
        color: Some(theme.palette().danger.base.color),
    };

    row![
        arrow_down().style(leecher_style),
        text(format!("{}", cant)).style(leecher_style)
    ]
    .spacing(4)
    .into()
}

fn trusted_badge() -> Element<'static, NyaaSearchMessage> {
    container(text("Trusted").size(11))
        .padding(tokens::BADGE_PADDING)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.palette().background.weak.color.into()),
            border: Border {
                radius: Radius::from(tokens::RADIUS_LARGE),
                color: theme.palette().success.weak.color,
                width: tokens::BORDER_THIN,
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
