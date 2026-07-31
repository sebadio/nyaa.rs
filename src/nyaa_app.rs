use crate::config::Config;
use crate::qbittorrent::Client;
use crate::ui::settings::{self, Settings};
use crate::ui::{Library, library};
use crate::ui::{custom_titlebar, main_view, sidebar};
use iced::Length::Fill;
use iced::time::{self, Duration};
use iced::widget::{column, row};
use iced::{Element, Task, window};
use iced::{Subscription, Theme};
use std::io::ErrorKind;

#[derive(Default)]
pub(crate) enum NyaaView {
    #[default]
    NyaaSearch,
    QtorLibrary(Library),
    Settings(Settings),
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
            NyaaView::NyaaSearch => ScreenKind::Search,
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
    Library(library::LibraryMessage),
    Settings(settings::SettingsMessage),
}

pub(crate) struct NyaaAppState {
    pub(crate) current_view: NyaaView,
    qbt_client: Client,
    config: Config,
}

impl NyaaAppState {
    pub(crate) fn new(config: Config) -> NyaaAppState {
        let qbt_client = Client::new(&config.qtor_url, &config.qtor_username, &config.qtor_pass)
            .expect("client");

        NyaaAppState {
            current_view: NyaaView::default(),
            qbt_client,
            config,
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
            NyaaMessage::Navigate(target) => {
                if self.current_view.kind() == target {
                    return Task::none();
                }
                log::debug!("Changed view to {target:?}");
                self.current_view = match target {
                    ScreenKind::Search => NyaaView::NyaaSearch,
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
                let config_updated =
                    matches!(settings_message, settings::SettingsMessage::UpdatedConfig);
                let NyaaView::Settings(settings) = &mut self.current_view else {
                    return Task::none();
                };

                let action = settings.update(settings_message, &mut self.config);

                if config_updated {
                    match Client::new(
                        &self.config.qtor_url,
                        &self.config.qtor_username,
                        &self.config.qtor_pass,
                    ) {
                        Ok(client) => self.qbt_client = client,
                        Err(e) => log::warn!("qbt client rebuild failed: {e}"),
                    }

                    if let Err(e) = self.config.save() {
                        log::warn!("failed to save config: {e}");
                    }
                }

                match action {
                    settings::Action::None => Task::none(),
                    settings::Action::Run(task) => task.map(NyaaMessage::Settings),
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
        }
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
