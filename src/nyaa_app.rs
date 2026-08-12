use crate::config::Config;
use crate::ui::main_view;
use crate::ui::settings::{self, Settings};
use crate::ui::widgets::modals::{self, download, modal};
use crate::ui::widgets::{Toast, ToastId, ToastKind, sidebar, status_bar, titlebar};
use crate::ui::{Library, library};
use crate::ui::{Search, search};
use crate::util::track_torrent;
use iced::Length::Fill;
use iced::time::Instant;
use iced::time::{self, Duration};
use iced::widget::{Stack, column, container, row};
use iced::{Animation, Element, Subscription, Task, Theme, window};
use log::{error, info};
use nyaa::NyaaAdapter;
use nyaa::NyaaAdapterError;
use nyaa::adapter::NyaaItemBytes;
use qbittorrent::{self, Client, Torrent, TorrentPostResponse};
use std::io::ErrorKind;

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
    ToggleSidebar,
    Tick,
    AnimationTick,
    Navigate(ScreenKind),
    Search(search::NyaaSearchMessage),
    Library(library::LibraryMessage),
    Settings(settings::SettingsMessage),
    TorrentQueued(TorrentPostResponse),
    DismissToast(ToastId),
    AddToast(Toast),
    TorrentAddFailed {
        error: qbittorrent::Error,
        original_hash: String,
    },
    DownloadProgress {
        name: String,
        progress: f32,
    },
    DownloadFinished(Result<Torrent, qbittorrent::Error>),
    TorrentDownloaded {
        options: download::Options,
        result: Result<NyaaItemBytes, NyaaAdapterError>,
    },
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
    notifications: Vec<Toast>,
    sidebar_animation: Animation<bool>,
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
            notifications: Vec::new(),
            sidebar_animation: Animation::new(true).very_quick(),
        }
    }

    pub(crate) fn view(&self) -> Element<'_, NyaaMessage> {
        let content = column![
            self.config.uses_custom_titlebar.then(titlebar),
            column![
                row![
                    sidebar(&self.sidebar_animation, self.current_view.kind()),
                    column![main_view(self)].width(Fill).padding(12)
                ]
                .spacing(12),
            ]
            .height(Fill),
            self.active_download.as_ref().map(status_bar)
        ];

        let mut layers = vec![content.into()];
        if let Some(active_modal) = &self.active_modal {
            layers.push(modal(
                active_modal.view().map(NyaaMessage::Modal),
                NyaaMessage::Modal(modals::Message::Cancel),
            ));
        }

        let notifications =
            container(column(self.notifications.iter().map(Toast::view)).spacing(8))
                .align_right(Fill)
                .align_top(Fill)
                .padding(16);

        layers.push(notifications.into());

        Stack::from_vec(layers).height(Fill).width(Fill).into()
    }

    pub(crate) fn update(&mut self, message: NyaaMessage) -> Task<NyaaMessage> {
        match message {
            NyaaMessage::Tick => {
                let now = Instant::now();
                self.notifications
                    .retain(|t| now.duration_since(t.created_at) < t.lifetime());
                Task::none()
            }

            NyaaMessage::AnimationTick => Task::none(),

            NyaaMessage::AddToast(toast) => {
                self.notifications.push(toast);
                Task::none()
            }

            NyaaMessage::DismissToast(id) => {
                self.notifications.retain(|t| t.id.ne(&id));
                Task::none()
            }

            NyaaMessage::ToggleSidebar => {
                let expanded = !self.sidebar_animation.value();
                self.sidebar_animation.go_mut(expanded, Instant::now());
                Task::none()
            }

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
                        }
                        modals::Event::PostDownloadSubmit(options) => {
                            let Some(modals::Modal::Download(modal)) = self.active_modal.take()
                            else {
                                return Task::none();
                            };

                            let client = self.nyaa_adapter.clone();
                            return Task::perform(
                                async move { client.download_torrent(modal.item).await },
                                move |result| NyaaMessage::TorrentDownloaded { options, result },
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
                        self.persist_config();
                        self.rebuild_qbt_client()
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
                    library::Action::Task(task) => task.map(NyaaMessage::Library),
                    library::Action::OpenPath(path) => {
                        if let Err(e) = open::that_detached(path) {
                            let toast = Toast::new()
                                .set_title("Failed to open")
                                .set_kind(ToastKind::Error);

                            let toast = match e.kind() {
                                ErrorKind::NotFound => {
                                    error!("File doesn't exist");
                                    toast.set_message("Does the file exist?")
                                }
                                _ => {
                                    error!("{}", e.kind());
                                    toast.set_message(format!("{}", e.kind()))
                                }
                            };

                            return Task::done(NyaaMessage::AddToast(toast));
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
                        Task::done(NyaaMessage::AddToast(Toast {
                            title: "Error".to_string(),
                            message: e.to_string(),
                            kind: ToastKind::Error,
                            ..Toast::default()
                        }))
                    }
                    search::Action::None => Task::none(),
                    search::Action::Task(task) => task.map(NyaaMessage::Search),
                    search::Action::OpenPostDownload(item) => {
                        self.active_modal =
                            Some(modals::Modal::Download(download::Modal::new(item)));
                        Task::none()
                    }
                }
            }

            NyaaMessage::TorrentDownloaded { options, result } => match result {
                Ok(nyaa_combo) => {
                    let title = nyaa_combo.item.title.clone();
                    self.active_download = Some(ActiveDownload {
                        name: nyaa_combo.item.title,
                        progress: 0.0,
                        open_on_finish: options.open_on_finish,
                    });

                    let client = self.qbt_client.clone();
                    let queue_task = Task::perform(
                        async move { client.queue_torrent(nyaa_combo.bytes).await },
                        move |result| match result {
                            Ok(post) => NyaaMessage::TorrentQueued(post),
                            Err(error) => NyaaMessage::TorrentAddFailed {
                                error,
                                original_hash: nyaa_combo.item.info_hash,
                            },
                        },
                    );

                    let toast = Toast::new()
                        .set_title("Starting Download")
                        .set_message(title)
                        .set_kind(ToastKind::Info);

                    Task::batch([Task::done(NyaaMessage::AddToast(toast)), queue_task])
                }
                Err(e) => {
                    error!("Failed to download torrent: {}", e);
                    Task::done(NyaaMessage::AddToast(Toast {
                        title: "Error".to_string(),
                        message: format!("Failed to download torrent: {}", e),
                        kind: ToastKind::Error,
                        ..Toast::default()
                    }))
                }
            },

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
                    let msg = "Torrent already in qBittorrent - tracking existing one".to_string();
                    info!("{}", msg);

                    let toast = Toast::new()
                        .set_title("Tracking existing torrent")
                        .set_message(msg)
                        .set_kind(ToastKind::Info);

                    let sip_task = Task::sip(
                        track_torrent(&self.qbt_client, original_hash),
                        |(name, progress)| NyaaMessage::DownloadProgress { name, progress },
                        NyaaMessage::DownloadFinished,
                    );

                    Task::batch([Task::done(NyaaMessage::AddToast(toast)), sip_task])
                }
                other => {
                    error!("queue failed: {other}");
                    self.active_download = None;
                    Task::done(NyaaMessage::AddToast(Toast {
                        title: "Error".to_string(),
                        message: format!("Failed to queue requested torrent: {}", other),
                        kind: ToastKind::Error,
                        ..Toast::default()
                    }))
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
                    (Err(e), _) => {
                        error!("tracking failed: {e}");
                        return Task::done(NyaaMessage::AddToast(Toast {
                            title: "Error tracking".to_string(),
                            message: format!("Tracking failed for torrent {}", e),
                            kind: ToastKind::Error,
                            ..Toast::default()
                        }));
                    }
                }
                Task::none()
            }
        }
    }

    fn rebuild_qbt_client(&mut self) -> Task<NyaaMessage> {
        match Client::new(
            &self.config.qtor_url,
            &self.config.qtor_username,
            &self.config.qtor_pass,
            Some(&self.config.qtor_save_path),
        ) {
            Ok(client) => {
                self.qbt_client = client;
                Task::none()
            }
            Err(e) => {
                error!("qbittorrent client rebuild failed: {e}");
                let toast = Toast::new()
                    .set_title("qBittorrent connection error")
                    .set_message(format!("Error connecting to qBittorrent {}", e))
                    .set_kind(ToastKind::Error);
                Task::done(NyaaMessage::AddToast(toast))
            }
        }
    }

    fn persist_config(&mut self) {
        if let Err(e) = self.config.save() {
            log::warn!("failed to save config: {e}");
        }
    }

    pub(crate) fn theme(&self) -> Option<Theme> {
        match &self.current_view {
            NyaaView::Settings(settings) => settings.config.theme.clone(),
            _ => self.config.theme.clone(),
        }
    }

    pub(crate) fn subscription(&self) -> Subscription<NyaaMessage> {
        let mut suscriptions = vec![time::every(Duration::from_secs(1)).map(|_| NyaaMessage::Tick)];

        if matches!(self.current_view, NyaaView::QtorLibrary(_)) && self.qbt_client.is_logged_in() {
            suscriptions.push(
                time::every(Duration::from_secs(2))
                    .map(|_| NyaaMessage::Library(library::LibraryMessage::Load)),
            );
        }

        if self.sidebar_animation.is_animating(Instant::now()) {
            suscriptions.push(window::frames().map(|_| NyaaMessage::AnimationTick));
        }

        Subscription::batch(suscriptions)
    }
}
