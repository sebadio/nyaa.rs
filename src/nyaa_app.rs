use crate::qbittorrent;
use crate::qbittorrent::{Client, Torrent};
use crate::ui::{custom_titlebar, main_view, sidebar};
use iced::Length::Fill;
use iced::Subscription;
use iced::time::{self, Duration};
use iced::widget::{column, row};
use iced::{Element, Task, window};
use log::{info, warn};

#[derive(Default)]
pub(crate) enum NyaaView {
    #[default]
    NyaaSearch,
    QtorLibrary,
    Settings,
}

#[derive(Debug, Clone)]
pub(crate) enum NyaaMessage {
    Exit,
    ToggleWindowMode,
    Minimize,
    Drag,
    NavigateToSearch,
    NavigateToLibrary,
    NavigateToSettings,
    LibraryQueryChanged(String),
    LoadTorrents,
    TorrentsLoaded(Result<Vec<Torrent>, qbittorrent::Error>),
    LibraryTorrentPressed(String),
}

pub(crate) struct NyaaAppState {
    pub(crate) current_view: NyaaView,
    pub(crate) library_query: String,
    pub(crate) library_torrents: Vec<Torrent>,
    qbt_client: Client,
}

impl NyaaAppState {
    pub(crate) fn new() -> NyaaAppState {
        NyaaAppState {
            current_view: NyaaView::default(),
            library_query: String::new(),
            qbt_client: Client::new("http://127.0.0.1:8080").expect("client"),
            library_torrents: Vec::new(),
        }
    }

    pub(crate) fn view(&self) -> Element<'_, NyaaMessage> {
        column![
            // custom_titlebar(),
            column![row![sidebar(), column![main_view(self)].width(Fill)].spacing(12),].padding(12)
        ]
        .into()
    }

    pub(crate) fn update(&mut self, message: NyaaMessage) -> Task<NyaaMessage> {
        match message {
            NyaaMessage::NavigateToSearch => self.current_view = NyaaView::NyaaSearch,
            NyaaMessage::NavigateToLibrary => {
                self.current_view = NyaaView::QtorLibrary;
                return Task::done(NyaaMessage::LoadTorrents);
            }
            NyaaMessage::NavigateToSettings => self.current_view = NyaaView::Settings,
            NyaaMessage::LibraryQueryChanged(text) => self.library_query = text,
            NyaaMessage::LoadTorrents => {
                let qbt = self.qbt_client.clone();
                return Task::perform(
                    async move {
                        qbt.login("iota", "cacatua123").await?;
                        qbt.get_torrents().await
                    },
                    NyaaMessage::TorrentsLoaded,
                );
            }
            NyaaMessage::TorrentsLoaded(Ok(list)) => self.library_torrents = list,
            NyaaMessage::TorrentsLoaded(Err(e)) => warn!("qbt error: {e:?}"),
            NyaaMessage::LibraryTorrentPressed(hash) => {
                info!("Pressed torrent: {}", hash);
                if let Some(t) = self.library_torrents.iter().find(|t| t.hash == hash) {
                    let _ = open::that(&t.content_path);
                }
            }
            NyaaMessage::Exit => return iced::exit(),
            NyaaMessage::ToggleWindowMode => {
                return window::latest().and_then(window::toggle_maximize);
            }
            NyaaMessage::Minimize => {
                return window::latest().and_then(|id| window::minimize(id, true));
            }
            NyaaMessage::Drag => return window::latest().and_then(window::drag),
        }

        Task::none()
    }

    pub(crate) fn subscription(&self) -> Subscription<NyaaMessage> {
        match self.current_view {
            NyaaView::QtorLibrary => {
                time::every(Duration::from_secs(1)).map(|_| NyaaMessage::LoadTorrents)
            }
            _ => Subscription::none(),
        }
    }
}
