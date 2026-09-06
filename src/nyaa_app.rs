use crate::config::Config;
use crate::nyaa_app::NyaaMessage::AddToast;
use crate::ui::main_view;
use crate::ui::settings::{self, Settings};
use crate::ui::widgets::modals::{self, delete_torrent, download, modal};
use crate::ui::widgets::{Toast, ToastId, ToastKind, sidebar, status_bar, titlebar};
use crate::ui::{Downloads, downloads};
use crate::ui::{Search, search};
use iced::Length::Fill;
use iced::time::Instant;
use iced::time::{self, Duration};
use iced::widget::{Stack, column, container, row};
use iced::{Animation, Element, Subscription, Task, Theme, window};
use log::{error, info};
use nyaa::NyaaAdapter;
use nyaa::NyaaAdapterError;
use nyaa::adapter::NyaaItemBytes;
use qbittorrent::{self, Client, Torrent};
use std::collections::HashMap;

pub(crate) enum NyaaView {
    NyaaSearch(Search),
    Downloads(Downloads),
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
    Downloads,
    Settings,
}

impl NyaaView {
    fn kind(&self) -> ScreenKind {
        match self {
            NyaaView::NyaaSearch(_) => ScreenKind::Search,
            NyaaView::Downloads(_) => ScreenKind::Downloads,
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
    TrackerTick,
    Navigate(ScreenKind),

    Search(search::NyaaSearchMessage),
    Downloads(downloads::DownloadsMessage),
    Settings(settings::SettingsMessage),

    DismissToast(ToastId),
    AddToast(Toast),
    Modal(modals::Message),

    TorrentQueued {
        hash: String,
        name: String,
        open_on_finish: bool,
    },
    TorrentAddFailed {
        error: qbittorrent::Error,
        original_hash: String,
    },
    TorrentsTracked(Result<Vec<Torrent>, qbittorrent::Error>),
    TorrentDownloaded {
        options: download::Options,
        result: Result<NyaaItemBytes, NyaaAdapterError>,
    },

    OpenTorrent(Torrent),
    OpenTorrentByHash(String),
}

const MISSING_POLLS_BEFORE_REMOVAL: u8 = 3;

#[derive(Debug, Default, Clone)]
pub(crate) struct ActiveDownload {
    pub name: String,
    pub progress: f32,
    pub hash: String,
    pub open_on_finish: bool,
    pub missing_polls: u8,
}

pub(crate) struct NyaaAppState {
    pub(crate) current_view: NyaaView,
    qbt_client: Client,
    nyaa_adapter: NyaaAdapter,
    config: Config,
    active_downloads: Vec<ActiveDownload>,
    tracking_request_in_flight: bool,
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
            active_downloads: Vec::new(),
            active_modal: None,
            notifications: Vec::new(),
            sidebar_animation: Animation::new(true).very_quick(),
            tracking_request_in_flight: false,
        }
    }

    pub(crate) fn view(&self) -> Element<'_, NyaaMessage> {
        let content = column![
            self.config.uses_custom_titlebar.then(titlebar),
            column![row![
                sidebar(&self.sidebar_animation, self.current_view.kind()),
                column![main_view(self), status_bar(&self.active_downloads)].width(Fill)
            ],]
            .height(Fill),
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

            NyaaMessage::TrackerTick => {
                if self.active_downloads.is_empty() || self.tracking_request_in_flight {
                    return Task::none();
                }

                let hashes = self
                    .active_downloads
                    .iter()
                    .map(|download| download.hash.clone())
                    .collect::<Vec<_>>();

                let client = self.qbt_client.clone();
                self.tracking_request_in_flight = true;

                Task::perform(
                    async move { client.get_torrents_by_hashes(&hashes).await },
                    NyaaMessage::TorrentsTracked,
                )
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
                        modals::Event::PostDeleteTorrentSubmit(options) => {
                            self.active_modal = None;
                            return Task::done(NyaaMessage::Downloads(
                                downloads::DownloadsMessage::ConfirmRemoveTorrent(options),
                            ));
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
                    ScreenKind::Downloads => NyaaView::Downloads(Downloads::new()),
                    ScreenKind::Settings => NyaaView::Settings(Settings::new(self.config.clone())),
                };
                match target {
                    ScreenKind::Downloads => {
                        Task::done(NyaaMessage::Downloads(downloads::DownloadsMessage::Load))
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
            NyaaMessage::Downloads(downloads_message) => {
                let NyaaView::Downloads(downloads) = &mut self.current_view else {
                    return Task::none();
                };
                match downloads.update(downloads_message, &self.qbt_client) {
                    downloads::Action::None => Task::none(),
                    downloads::Action::Task(task) => task.map(NyaaMessage::Downloads),
                    downloads::Action::ShowError(err) => Task::done(AddToast(
                        Toast::new()
                            .set_kind(ToastKind::Error)
                            .set_message(err)
                            .set_title("Error"),
                    )),
                    downloads::Action::TorrentRemoved(hash) => {
                        self.active_downloads
                            .retain(|download| download.hash.ne(&hash));
                        Task::none()
                    }
                    downloads::Action::OpenTorrent(torrent) => {
                        Task::done(NyaaMessage::OpenTorrent(torrent))
                    }
                    downloads::Action::OpenDeleteTorrent(hash) => {
                        self.active_modal = Some(modals::Modal::DeleteTorrent(
                            delete_torrent::Modal::new(hash),
                        ));
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
                    let original_hash = nyaa_combo.item.info_hash;

                    let toast = Toast::new()
                        .set_title("Starting Download")
                        .set_message(title.clone())
                        .set_kind(ToastKind::Info);

                    let client = self.qbt_client.clone();
                    let queue_task = Task::perform(
                        async move { client.queue_torrent(nyaa_combo.bytes).await },
                        move |result| match result {
                            Ok(post) => NyaaMessage::TorrentQueued {
                                hash: post
                                    .added_torrent_ids
                                    .first()
                                    .expect("queue_torrent guarantees a non empty list")
                                    .clone(),
                                name: title,
                                open_on_finish: options.open_on_finish,
                            },
                            Err(error) => NyaaMessage::TorrentAddFailed {
                                error,
                                original_hash: original_hash,
                            },
                        },
                    );

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

            NyaaMessage::TorrentQueued {
                hash,
                name,
                open_on_finish,
            } => {
                self.active_downloads.push(ActiveDownload {
                    hash,
                    name,
                    progress: 0.0,
                    open_on_finish,
                    ..Default::default()
                });

                Task::none()
            }

            NyaaMessage::TorrentAddFailed {
                error,
                original_hash,
            } => match error {
                qbittorrent::Error::AlreadyExists() => {
                    let msg = "Torrent already in qBittorrent".to_string();
                    info!("{}", msg);

                    let toast = Toast::new()
                        .set_title("Not tracking existing torrent")
                        .set_message(msg)
                        .set_kind(ToastKind::Info);

                    Task::batch([
                        Task::done(NyaaMessage::AddToast(toast)),
                        Task::done(NyaaMessage::OpenTorrentByHash(original_hash)),
                    ])
                }
                other => {
                    error!("queue failed: {other}");

                    Task::done(NyaaMessage::AddToast(Toast {
                        title: "Error".to_string(),
                        message: format!("Failed to queue requested torrent: {}", other),
                        kind: ToastKind::Error,
                        ..Toast::default()
                    }))
                }
            },

            NyaaMessage::TorrentsTracked(Ok(torrents)) => {
                self.tracking_request_in_flight = false;
                let torrents_by_hash: HashMap<_, _> = torrents
                    .into_iter()
                    .map(|torrent| (torrent.hash.clone(), torrent))
                    .collect();

                let mut tasks = Vec::new();
                self.active_downloads.retain_mut(|download| {
                    let Some(torrent) = torrents_by_hash.get(&download.hash) else {
                        download.missing_polls = download.missing_polls.saturating_add(1);
                        return download.missing_polls < MISSING_POLLS_BEFORE_REMOVAL;
                    };
                    download.missing_polls = 0;
                    download.name = torrent.name.clone();
                    download.progress = torrent.progress;
                    if torrent.is_complete() {
                        if download.open_on_finish {
                            tasks.push(Task::done(NyaaMessage::OpenTorrent(torrent.clone())));
                        }
                        false
                    } else {
                        true
                    }
                });
                Task::batch(tasks)
            }

            NyaaMessage::TorrentsTracked(Err(error)) => {
                error!("tracking failed: {error}");
                self.tracking_request_in_flight = false;

                return Task::done(NyaaMessage::AddToast(
                    Toast::new()
                        .set_title("Error tracking")
                        .set_message(error.to_string())
                        .set_kind(ToastKind::Error),
                ));
            }

            NyaaMessage::OpenTorrent(torrent) => {
                if let Err(e) = open::that_detached(&torrent.content_path) {
                    error!("open failed: {e}");
                    return Task::done(NyaaMessage::AddToast(
                        Toast::new()
                            .set_kind(ToastKind::Error)
                            .set_title("Error opening file")
                            .set_message(e.to_string()),
                    ));
                }
                Task::none()
            }

            NyaaMessage::OpenTorrentByHash(hash) => {
                let qbt = self.qbt_client.clone();
                let hashes = vec![hash];

                Task::perform(
                    async move { qbt.get_torrents_by_hashes(&hashes).await },
                    |res| match res {
                        Ok(res) => match res.first() {
                            Some(torrent) => NyaaMessage::OpenTorrent(torrent.clone()),
                            None => NyaaMessage::Tick,
                        },
                        Err(e) => {
                            error!("{}", e);

                            NyaaMessage::AddToast(
                                Toast::new()
                                    .set_title("Error opening file")
                                    .set_message(e.to_string())
                                    .set_kind(ToastKind::Error),
                            )
                        }
                    },
                )
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
        // FIX -> Maybe I can do something like "self.current_view.tick() and push it to the suscription?"
        if matches!(self.current_view, NyaaView::Downloads(_)) && self.qbt_client.is_logged_in() {
            suscriptions.push(
                time::every(Duration::from_secs(1))
                    .map(|_| NyaaMessage::Downloads(downloads::DownloadsMessage::Load)),
            );
        }

        if self.sidebar_animation.is_animating(Instant::now()) {
            suscriptions.push(window::frames().map(|_| NyaaMessage::AnimationTick));
        }

        if !self.active_downloads.is_empty() && !self.tracking_request_in_flight {
            suscriptions
                .push(time::every(Duration::from_secs(1)).map(|_| NyaaMessage::TrackerTick));
        }

        Subscription::batch(suscriptions)
    }
}
