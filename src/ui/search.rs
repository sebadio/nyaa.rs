use iced::Length::Fill;
use iced::widget::{Text, button, column, pick_list, row, scrollable, space, text, text_input};
use iced::{Alignment, Element, Task, Theme};
use iced_fonts::lucide::search;
use log::{error, info};
use nyaa::filter::NyaaFilter;
use nyaa::request::NyaaRequest;
use nyaa::{NyaaAdapter, NyaaAdapterError, NyaaCategory, NyaaItem};
use thiserror::Error;

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

// FIX: route these to the main state and handle once popups/messages are done
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
}

impl Search {
    pub(crate) fn new() -> Self {
        Self {
            query: String::new(),
            category: NyaaCategory::default(),
            results: Vec::new(),
            filter: NyaaFilter::default(),
            has_searched: false,
        }
    }

    pub(crate) fn view<'a>(&self) -> Element<'a, NyaaSearchMessage> {
        let search_row = row![
            pick_list(
                NyaaFilter::ALL,
                Some(self.filter),
                NyaaSearchMessage::FilterUpdated
            ),
            space().width(12),
            pick_list(
                NyaaCategory::ALL,
                Some(self.category),
                NyaaSearchMessage::CategoryUpdated
            ),
            space().width(12),
            text_input("Let's search nyaa!", &self.query)
                .on_input(NyaaSearchMessage::QueryUpdated)
                .on_submit(NyaaSearchMessage::Search)
                .width(Fill),
            space().width(12),
            button(search()).on_press(NyaaSearchMessage::Search)
        ]
        .width(Fill);

        let content: Element<'a, NyaaSearchMessage> = if self.results.is_empty() {
            column![
                text(if self.has_searched {
                    "No results"
                } else {
                    "Let's search something up!"
                })
                .width(Fill)
                .height(Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .size(24)
            ]
            .into()
        } else {
            scrollable(self.results.iter().fold(column![], |col, item| {
                col.push(self.item_element(item.clone()))
            }))
            .width(Fill)
            .into()
        };

        column![
            text("Nyaa search").width(Fill),
            search_row,
            space().height(12),
            content
        ]
        .into()
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

            NyaaSearchMessage::SearchResults(result) => {
                self.has_searched = true;
                match result {
                    Ok(items) => {
                        self.results = items;
                        Action::None
                    }
                    Err(e) => {
                        error!("Search failed: {e}");
                        Action::ShowError(SearchViewError::FailedSearch(format!(
                            "Search has failed! {}",
                            e
                        )))
                    }
                }
            }

            NyaaSearchMessage::DownloadTorrent(item) => Action::OpenPostDownload(item),
        }
    }

    // FIX: Move this to a shared ui appeareance folder or something?
    fn item_element<'a>(&self, item: NyaaItem) -> Element<'a, NyaaSearchMessage> {
        let subtitle: Text = text(format!("Seeders: {} - Size: {}", item.seeders, item.size));

        button(column![
            text(item.title.to_string()),
            subtitle,
            space().height(12)
        ])
        .on_press(NyaaSearchMessage::DownloadTorrent(item))
        .style(|theme: &Theme, status| {
            let palette = theme.extended_palette();
            button::Style {
                background: match status {
                    button::Status::Hovered => Some(palette.background.base.color.into()),
                    _ => None,
                },
                text_color: match status {
                    button::Status::Hovered => palette.primary.strong.color,
                    _ => palette.background.base.text,
                },
                ..button::text(theme, status)
            }
        })
        .into()
    }
}
