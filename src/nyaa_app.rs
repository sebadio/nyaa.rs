use crate::config::Config;
use crate::ui::settings::{self, Settings};
use crate::ui::widgets::modals::{self, modal, post_download};
use crate::ui::{Library, library};
use crate::ui::{Search, search};
use crate::ui::{custom_titlebar, main_view, sidebar};
use crate::util::truncate_with_ellipsis;
use iced::Length::{self, Fill};
use iced::task::{Sipper, sipper};
use iced::time::{self, Duration};
use iced::widget::{column, progress_bar, row, text};
use iced::{Element, Subscription, Task, Theme, window};
use log::{error, info};
use nyaa::NyaaAdapter;
use nyaa::adapter::NyaaItemBytes;
use qbittorrent::{self, Client, Error, Torrent, TorrentPostResponse};
use std::io::ErrorKind;
use tokio::time::sleep;

pub(crate) enum NyaaView {
    NyaaSearch(Search),
    QtorLibrary(Library),
    Settings(Settings),
}

impl Default for NyaaView {
    fn default() -> Self {
        Self::NyaaSearch(Search::new())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScreenKind {
    Search,
    Library,
    Settings,
}

impl NyaaView {
    fn kind(&self) -> ScreenKind {
        match self {
            NyaaView::NyaaSearch(_) => ScreenKind::Search,
            NyaaView::QtorLibrary(_) => ScreenKind::Library,
            NyaaView::Settings(_) => ScreenKind::Settings,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum NyaaMessage {
    Exit,
    ToggleWindowMode,
    Minimize,
    Drag,
    Navigate(ScreenKind),
    Search(search::NyaaSearchMessage),
    Library(library::LibraryMessage),
    Settings(settings::SettingsMessage),
    TorrentQueued(TorrentPostResponse),
    TorrentAddFailed {
        error: qbittorrent::Error,
        original_hash: String,
    },
    DownloadProgress {
        name: String,
        progress: f32,
    },
    DownloadFinished(Result<Torrent, qbittorrent::Error>),
    Modal(modals::Message),
}

pub(crate) struct ActiveDownload {
    pub name: String,
    pub progress: f32,
    pub open_on_finish: bool,
}

pub(crate) struct NyaaAppState {
    pub(crate) current_view: NyaaView,
    qbt_client: Client,
    nyaa_adapter: NyaaAdapter,
    config: Config,
    active_download: Option<ActiveDownload>,
    active_modal: Option<modals::Modal>,
    pending_download: Option<NyaaItemBytes>,
}

impl NyaaAppState {
    pub(crate) fn new(config: Config) -> NyaaAppState {
        let qbt_client = Client::new(
            &config.qtor_url,
            &config.qtor_username,
            &config.qtor_pass,
            Some(&config.qtor_save_path),
        )
        .expect("client");

        let nyaa_adapter = NyaaAdapter::new().expect("Nyaa adapter should not fail");

        NyaaAppState {
            current_view: NyaaView::default(),
            qbt_client,
            nyaa_adapter,
            config,
            active_download: None,
            active_modal: None,
            pending_download: None,
        }
    }

    pub(crate) fn view(&self) -> Element<'_, NyaaMessage> {
        let content = column![
            self.config.uses_custom_titlebar.then(custom_titlebar),
            column![row![sidebar(), column![main_view(self)].width(Fill)].spacing(12),]
                .padding(12)
                .height(Fill),
            self.active_download.as_ref().map(Self::status_bar)
        ];

        if let Some(active_modal) = &self.active_modal {
            modal(
                content,
                active_modal.view().map(NyaaMessage::Modal),
                NyaaMessage::Modal(modals::Message::Cancel),
            )
        } else {
            content.into()
        }
    }

    pub(crate) fn update(&mut self, message: NyaaMessage) -> Task<NyaaMessage> {
        match message {
            NyaaMessage::Modal(message) => {
                let Some(active_modal) = &mut self.active_modal else {
                    return Task::none();
                };

                let (task, event) = active_modal.update(message);

                if let Some(event) = event {
                    match event {
                        modals::Event::CloseModal => {
                            self.active_modal = None;
                        }

                        modals::Event::PostModalCancel => {
                            self.active_modal = None;
                            self.pending_download = None
                        }
                        modals::Event::PostDownloadSubmit(options) => {
                            self.active_modal = None;

                            let Some(nyaa_combo) = self.pending_download.take() else {
                                return Task::none();
                            };

                            self.active_download = Some(ActiveDownload {
                                name: nyaa_combo.item.title,
                                progress: 0.0,
                                open_on_finish: options.open_on_finish,
                            });

                            let client = self.qbt_client.clone();
                            return Task::perform(
                                async move { client.queue_torrent(nyaa_combo.bytes).await },
                                move |result| match result {
                                    Ok(post) => NyaaMessage::TorrentQueued(post),
                                    Err(error) => NyaaMessage::TorrentAddFailed {
                                        error,
                                        original_hash: nyaa_combo.item.info_hash,
                                    },
                                },
                            );
                        }

                        modals::Event::SaveSettingsDecision(decision) => {
                            self.active_modal = None;

                            // Handle save/discard decision here.
                        }
                    }
                }

                task.map(NyaaMessage::Modal)
            }

            NyaaMessage::Navigate(target) => {
                if self.current_view.kind() == target {
                    return Task::none();
                }
                log::info!("Changed view to {target:?}");
                self.current_view = match target {
                    ScreenKind::Search => NyaaView::NyaaSearch(Search::new()),
                    ScreenKind::Library => NyaaView::QtorLibrary(Library::new()),
                    ScreenKind::Settings => NyaaView::Settings(Settings::new(self.config.clone())),
                };
                match target {
                    ScreenKind::Library => {
                        Task::done(NyaaMessage::Library(library::LibraryMessage::Load))
                    }
                    _ => Task::none(),
                }
            }
            NyaaMessage::Settings(settings_message) => {
                let NyaaView::Settings(settings) = &mut self.current_view else {
                    return Task::none();
                };

                match settings.update(settings_message, &mut self.config) {
                    settings::Action::None => Task::none(),
                    settings::Action::Task(task) => task.map(NyaaMessage::Settings),
                    settings::Action::ApplyConfig => {
                        self.rebuild_qbt_client();
                        self.persist_config();
                        Task::none()
                    }
                }
            }
            NyaaMessage::Exit => iced::exit(),
            NyaaMessage::ToggleWindowMode => window::latest().and_then(window::toggle_maximize),
            NyaaMessage::Minimize => window::latest().and_then(|id| window::minimize(id, true)),
            NyaaMessage::Drag => window::latest().and_then(window::drag),
            NyaaMessage::Library(library_message) => {
                let NyaaView::QtorLibrary(library) = &mut self.current_view else {
                    return Task::none();
                };
                match library.update(library_message, &self.qbt_client) {
                    library::Action::None => Task::none(),
                    library::Action::Run(task) => task.map(NyaaMessage::Library),
                    library::Action::OpenPath(path) => {
                        if let Err(e) = open::that(path) {
                            match e.kind() {
                                ErrorKind::NotFound => log::error!("File doesn't exist"),
                                _ => log::error!("{}", e.kind()),
                            }
                        }
                        Task::none()
                    }
                }
            }

            NyaaMessage::Search(search_message) => {
                let NyaaView::NyaaSearch(search) = &mut self.current_view else {
                    return Task::none();
                };

                match search.update(search_message, &self.nyaa_adapter) {
                    search::Action::ShowError(e) => {
                        error!("{}", e);
                        Task::none()
                    }
                    search::Action::None => Task::none(),
                    search::Action::Run(task) => task.map(NyaaMessage::Search),
                    search::Action::AddToQbt(nyaa_combo) => {
                        self.pending_download = Some(nyaa_combo);
                        self.active_modal =
                            Some(modals::Modal::PostDownload(post_download::Modal::new()));
                        Task::none()
                    }
                }
            }

            NyaaMessage::TorrentQueued(res) => {
                let hash = res
                    .added_torrent_ids
                    .first()
                    .expect("queue_torrent guarantees a non empty list")
                    .to_string();

                Task::sip(
                    track_torrent(&self.qbt_client, hash),
                    |(name, progress)| NyaaMessage::DownloadProgress { name, progress },
                    NyaaMessage::DownloadFinished,
                )
            }

            NyaaMessage::TorrentAddFailed {
                error,
                original_hash,
            } => match error {
                qbittorrent::Error::AlreadyExists() => {
                    info!("Torrent already in qBittorrent — tracking existing one");
                    Task::sip(
                        track_torrent(&self.qbt_client, original_hash),
                        |(name, progress)| NyaaMessage::DownloadProgress { name, progress },
                        NyaaMessage::DownloadFinished,
                    )
                }
                other => {
                    error!("queue failed: {other}");
                    self.active_download = None;
                    self.pending_download = None;
                    Task::none()
                }
            },

            NyaaMessage::DownloadProgress { name, progress } => {
                if let Some(dl) = &mut self.active_download {
                    dl.name = name;
                    dl.progress = progress;
                }
                Task::none()
            }

            NyaaMessage::DownloadFinished(res) => {
                let dl = self.active_download.take();
                match (res, dl) {
                    (Ok(torrent), Some(dl)) if dl.open_on_finish => {
                        if let Err(e) = open::that_detached(&torrent.content_path) {
                            error!("open failed: {e}");
                        }
                    }
                    (Ok(_), _) => {}
                    (Err(e), _) => error!("tracking failed: {e}"),
                }
                Task::none()
            }
        }
    }

    fn rebuild_qbt_client(&mut self) {
        match Client::new(
            &self.config.qtor_url,
            &self.config.qtor_username,
            &self.config.qtor_pass,
            Some(&self.config.qtor_save_path),
        ) {
            Ok(client) => self.qbt_client = client,
            Err(e) => error!("qbittorrent client rebuild failed: {e}"),
        }
    }

    fn persist_config(&mut self) {
        if let Err(e) = self.config.save() {
            log::warn!("failed to save config: {e}");
        }
    }

    fn status_bar(download: &ActiveDownload) -> Element<'_, NyaaMessage> {
        let download_text = format!(
            "Downloading: {}",
            truncate_with_ellipsis(&download.name, 60)
        );

        row![
            text(download_text).width(Fill),
            progress_bar(0.0..=1.0, download.progress).length(Length::Fixed(200.0)),
            text(format!("{:.0}%", download.progress * 100.0)),
        ]
        .padding(8)
        .height(40)
        .into()
    }

    pub(crate) fn theme(&self) -> Option<Theme> {
        match &self.current_view {
            NyaaView::Settings(settings) => settings.config.theme.clone(),
            _ => self.config.theme.clone(),
        }
    }

    pub(crate) fn subscription(&self) -> Subscription<NyaaMessage> {
        match &self.current_view {
            NyaaView::QtorLibrary(_) if self.qbt_client.is_logged_in() => {
                time::every(Duration::from_secs(2))
                    .map(|_| NyaaMessage::Library(library::LibraryMessage::Load))
            }
            _ => Subscription::none(),
        }
    }
}

pub fn track_torrent(
    client: &Client,
    hash: String,
) -> impl Sipper<Result<Torrent, qbittorrent::Error>, (String, f32)> + 'static {
    let client = client.clone();

    sipper(move |mut sender| async move {
        loop {
            match client.get_torrent_by_hash(&hash).await {
                Ok(t) if t.progress >= 1.0 => return Ok(t),
                Ok(t) => {
                    sender.send((t.name, t.progress)).await;
                }
                Err(e @ Error::TorrentNotFound(_)) => return Err(e),
                Err(e) => error!("{e}"),
            }

            sleep(Duration::from_secs(1)).await;
        }
    })
}
