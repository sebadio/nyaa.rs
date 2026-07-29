use crate::qbittorrent::{self, Client, Torrent};
use chrono::{DateTime, Local, Utc};
use humansize::{DECIMAL, format_size};
use iced::Length::{Fill, FillPortion};
use iced::widget::scrollable::Scrollbar;
use iced::widget::text::Wrapping;
use iced::widget::{button, column, progress_bar, row, scrollable, space, table, text, text_input};
use iced::{Element, Font, Task, Theme, font};
use iced_fonts::lucide::refresh_cw;
use std::format;
use std::path::Path;

pub(crate) struct Library {
    pub(crate) query: String,
    pub(crate) torrents: Vec<Torrent>,
}

#[derive(Debug, Clone)]
pub(crate) enum LibraryMessage {
    QueryChanged(String),
    Load,
    Loaded(Result<Vec<Torrent>, qbittorrent::Error>),
    TorrentPressed(String),
}

pub(crate) enum Action {
    None,
    Run(Task<LibraryMessage>),
    OpenPath(String),
}

impl Library {
    pub(crate) fn new() -> Self {
        Self {
            query: String::new(),
            torrents: Vec::new(),
        }
    }

    pub fn update(&mut self, message: LibraryMessage, client: &Client) -> Action {
        match message {
            LibraryMessage::QueryChanged(q) => {
                self.query = q;
                Action::None
            }
            LibraryMessage::Load => {
                let qbt = client.clone();
                Action::Run(Task::perform(
                    async move { qbt.get_torrents().await },
                    LibraryMessage::Loaded,
                ))
            }
            LibraryMessage::Loaded(Ok(list)) => {
                self.torrents = list;
                Action::None
            }
            LibraryMessage::Loaded(Err(e)) => {
                log::warn!("qbt: {e:?}");
                Action::None
            }
            LibraryMessage::TorrentPressed(hash) => {
                match self.torrents.iter().find(|t| t.hash == hash) {
                    Some(t) => Action::OpenPath(t.content_path.clone()),
                    None => Action::None,
                }
            }
        }
    }

    pub(crate) fn view<'a>(&self) -> Element<'a, LibraryMessage> {
        let filtered_torrents = filter_library_torrents(&self.query, &self.torrents);

        let column_header_font = Font {
            weight: font::Weight::Bold,
            ..Font::default()
        };

        let columns = [
            table::column(text("Name").font(column_header_font), |t: Torrent| {
                library_table_button(t)
            })
            .width(FillPortion(6)),
            table::column(text("Size").font(column_header_font), |t: Torrent| {
                text(format_size(t.size, DECIMAL)).wrapping(Wrapping::None)
            })
            .width(FillPortion(1)),
            table::column(text("Seeders").font(column_header_font), |t: Torrent| {
                text(format!("{} ({})", t.num_seeds, t.num_complete)).wrapping(Wrapping::None)
            })
            .width(FillPortion(1)),
            table::column(text("Leechs").font(column_header_font), |t: Torrent| {
                text(format!("{} ({})", t.num_leechs, t.num_incomplete)).wrapping(Wrapping::None)
            })
            .width(FillPortion(1)),
            table::column(text("Progress").font(column_header_font), |t: Torrent| {
                column![
                    progress_bar(0.0..=100.0, t.progress * 100.0),
                    text(format!("{:.0}%", t.progress * 100.0))
                ]
            })
            .width(FillPortion(1)),
            table::column(text("Added On").font(column_header_font), |t: Torrent| {
                let dt_utc = DateTime::<Utc>::from_timestamp(t.added_on, 0).unwrap();
                let dt_local: DateTime<Local> = dt_utc.into();
                text(format!("{}", dt_local.format("%y-%m-%d %H:%M:%S")))
            })
            .width(FillPortion(1)),
        ];

        const TOOLBAR_HEIGHT: f32 = 40.0;
        let header_search = row![
            text_input("Search for downloaded torrents here", &self.query)
                .on_input(LibraryMessage::QueryChanged)
                .line_height(2.0)
                .padding(8)
                .width(Fill),
            space().width(12),
            button(
                refresh_cw()
                    .align_x(iced::Alignment::Center)
                    .align_y(iced::Alignment::Center)
            )
            .height(TOOLBAR_HEIGHT)
            .width(TOOLBAR_HEIGHT)
            .on_press(LibraryMessage::Load)
        ]
        .height(TOOLBAR_HEIGHT);

        column![
            header_search,
            space().height(20),
            row![
                scrollable(table(columns, filtered_torrents.iter().cloned()))
                    .width(Fill)
                    .height(Fill)
                    .direction(scrollable::Direction::Vertical(
                        Scrollbar::new().width(8).scroller_width(8).spacing(5) // ← embeds it: always shown, reserves space, doesn't float
                    ))
            ]
            .height(Fill)
        ]
        .into()
    }
}

fn library_table_button(torrent: Torrent) -> Element<'static, LibraryMessage> {
    let hash = torrent.hash.clone();
    button(text(torrent.name).wrapping(Wrapping::WordOrGlyph))
        .style(|theme: &Theme, status| {
            let palette = theme.extended_palette();
            button::Style {
                background: match status {
                    button::Status::Hovered => Some(palette.background.weak.color.into()),
                    _ => None,
                },
                text_color: match status {
                    button::Status::Hovered => palette.danger.strong.color,
                    _ => palette.background.base.text,
                },
                ..button::text(theme, status)
            }
        })
        .width(Fill)
        .on_press(LibraryMessage::TorrentPressed(hash))
        .into()
}

fn normalize(s: &str) -> String {
    s.to_lowercase().replace(['.', '_', '-'], " ")
}

fn is_video(name: &str) -> bool {
    const VIDEO_EXTS: [&str; 5] = ["mkv", "mp4", "avi", "mov", "webm"];
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| VIDEO_EXTS.iter().any(|v| e.eq_ignore_ascii_case(v)))
        .unwrap_or(false)
}

fn filter_library_torrents(query: &str, torrents: &[Torrent]) -> Vec<Torrent> {
    let normalized_query = normalize(query);
    let words: Vec<&str> = normalized_query.split_whitespace().collect();

    torrents
        .iter()
        .filter(|t| {
            let normalized_name = normalize(&t.name);
            is_video(&t.content_path) && words.iter().all(|w| normalized_name.contains(w))
        })
        .cloned()
        .collect()
}
