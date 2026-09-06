use crate::appearance::{self, tokens};
use crate::ui::widgets::download_item::download_item;
use crate::ui::widgets::modals::delete_torrent;
use iced::Length::Fill;
use iced::widget::scrollable::Scrollbar;
use iced::widget::{button, column, row, scrollable, text_input};
use iced::{Element, Task, alignment};
use iced_fonts::lucide::refresh_cw;
use qbittorrent::{self, Client, Torrent};
use std::path::Path;

pub(crate) struct Downloads {
    pub(crate) query: String,
    pub(crate) torrents: Vec<Torrent>,
}

#[derive(Debug, Clone)]
pub(crate) enum DownloadsMessage {
    QueryChanged(String),
    Load,
    Loaded(Result<Vec<Torrent>, qbittorrent::Error>),
    TorrentPressed(Torrent),

    PauseTorrent(String),
    ResumeTorrent(String),
    RemoveTorrent(String),
    ConfirmRemoveTorrent(delete_torrent::Options),

    PauseResult(Result<(), qbittorrent::Error>),
    ResumeResult(Result<(), qbittorrent::Error>),
    RemoveResult(Result<String, qbittorrent::Error>),
}

pub(crate) enum Action {
    None,
    Task(Task<DownloadsMessage>),
    ShowError(String),
    OpenTorrent(Torrent),
    OpenDeleteTorrent(String),
    TorrentRemoved(String),
}

impl Downloads {
    pub(crate) fn new() -> Self {
        Self {
            query: String::new(),
            torrents: Vec::new(),
        }
    }

    pub fn update(&mut self, message: DownloadsMessage, client: &Client) -> Action {
        match message {
            DownloadsMessage::PauseResult(Ok(_)) | DownloadsMessage::ResumeResult(Ok(_)) => {
                Action::None
            }
            DownloadsMessage::RemoveResult(Ok(hash)) => Action::TorrentRemoved(hash),

            DownloadsMessage::PauseResult(Err(e))
            | DownloadsMessage::RemoveResult(Err(e))
            | DownloadsMessage::ResumeResult(Err(e)) => Action::ShowError(e.to_string()),

            DownloadsMessage::PauseTorrent(hash) => {
                log::info!("Paused {}", hash);
                let qbt = client.clone();
                Action::Task(Task::perform(
                    async move { qbt.pause_torrent(&hash).await },
                    DownloadsMessage::PauseResult,
                ))
            }

            DownloadsMessage::ResumeTorrent(hash) => {
                log::info!("Resumed {}", hash);
                let qbt = client.clone();
                Action::Task(Task::perform(
                    async move { qbt.resume_torrent(&hash).await },
                    DownloadsMessage::ResumeResult,
                ))
            }

            DownloadsMessage::RemoveTorrent(hash) => Action::OpenDeleteTorrent(hash),

            DownloadsMessage::ConfirmRemoveTorrent(options) => {
                let hash = options.hash;
                let delete_files = options.delete_files;

                log::info!("Removing {} (delete files: {})", hash, delete_files);
                let qbt = client.clone();
                Action::Task(Task::perform(
                    async move {
                        qbt.remove_torrent(&hash, delete_files)
                            .await
                            .map(|_| hash.clone())
                    },
                    DownloadsMessage::RemoveResult,
                ))
            }

            DownloadsMessage::QueryChanged(q) => {
                self.query = q;
                Action::None
            }

            DownloadsMessage::Load => {
                let qbt = client.clone();
                Action::Task(Task::perform(
                    async move { qbt.get_torrents().await },
                    DownloadsMessage::Loaded,
                ))
            }

            DownloadsMessage::Loaded(Ok(list)) => {
                self.torrents = list;
                Action::None
            }

            DownloadsMessage::Loaded(Err(e)) => {
                log::error!("qbt: {e:?}");
                Action::ShowError(e.to_string())
            }

            DownloadsMessage::TorrentPressed(torrent) => Action::OpenTorrent(torrent),
        }
    }

    pub(crate) fn view(&self) -> Element<'_, DownloadsMessage> {
        let filtered_torrents = filter_torrents(&self.query, &self.torrents);

        let header_search = row![
            text_input("Search for downloaded torrents here", &self.query)
                .on_input(DownloadsMessage::QueryChanged)
                .size(tokens::INPUT_SIZE)
                .line_height(tokens::INPUT_LINE_HEIGHT)
                .padding(tokens::INPUT_PADDING)
                .style(appearance::text_input::primary)
                .align_x(alignment::Horizontal::Center)
                .width(Fill),
            button(
                refresh_cw()
                    .align_x(iced::Alignment::Center)
                    .align_y(iced::Alignment::Center)
            )
            .style(appearance::button::secondary)
            .height(tokens::BUTTON_SIZE)
            .width(tokens::BUTTON_SIZE)
            .padding(tokens::BTN_PADDING)
            .on_press(DownloadsMessage::Load)
        ]
        .spacing(tokens::SPACING_BASE)
        .height(tokens::BUTTON_SIZE);

        column![
            header_search,
            row![
                scrollable(
                    column(filtered_torrents.iter().map(download_item))
                        .spacing(tokens::SPACING_BASE)
                )
                .width(Fill)
                .height(Fill)
                .direction(scrollable::Direction::Vertical(
                    Scrollbar::new().width(8).scroller_width(8).spacing(5) // ← embeds it: always shown, reserves space, doesn't float
                ))
            ]
            .height(Fill)
        ]
        .spacing(tokens::SPACING_LARGE)
        .into()
    }
}

fn normalize(s: &str) -> String {
    s.to_lowercase().replace(['.', '_', '-'], " ")
}

const VIDEO_EXTENSIONS: &[&str] = &["mkv", "mp4", "avi", "mov", "webm", "m4v", "ts", "m2ts"];

fn is_video(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            VIDEO_EXTENSIONS
                .iter()
                .any(|video_ext| ext.eq_ignore_ascii_case(video_ext))
        })
}

fn filter_torrents(query: &str, torrents: &[Torrent]) -> Vec<Torrent> {
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
