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

#[derive(Default)]
pub(crate) enum NyaaView {
    #[default]
    NyaaSearch,
    QtorLibrary(Library),
    Settings(Settings),
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
            NyaaMessage::NavigateToSearch => {
                self.current_view = NyaaView::NyaaSearch;
            }
            NyaaMessage::NavigateToLibrary => {
                self.current_view = NyaaView::QtorLibrary(Library::new());
                return Task::done(NyaaMessage::Library(library::LibraryMessage::Load));
            }
            NyaaMessage::NavigateToSettings => {
                self.current_view = NyaaView::Settings(Settings::new(self.config.clone()));
                return Task::none();
            }
            NyaaMessage::Settings(settings_message) => {
                let config_updated = matches!(
                    settings_message,
                    settings::SettingsMessage::UpdatedConfig(_)
                );
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

                return match action {
                    settings::Action::None => Task::none(),
                    settings::Action::Run(task) => task.map(NyaaMessage::Settings),
                };
            }
            NyaaMessage::Exit => return iced::exit(),
            NyaaMessage::ToggleWindowMode => {
                return window::latest().and_then(window::toggle_maximize);
            }
            NyaaMessage::Minimize => {
                return window::latest().and_then(|id| window::minimize(id, true));
            }
            NyaaMessage::Drag => return window::latest().and_then(window::drag),
            NyaaMessage::Library(library_message) => {
                let NyaaView::QtorLibrary(library) = &mut self.current_view else {
                    return Task::none();
                };
                return match library.update(library_message, &self.qbt_client) {
                    library::Action::None => Task::none(),
                    library::Action::Run(task) => task.map(NyaaMessage::Library),
                    library::Action::OpenPath(path) => {
                        let _ = open::that(path);
                        Task::none()
                    }
                };
            }
        }

        Task::none()
    }

    pub(crate) fn theme(&self) -> Option<Theme> {
        self.config.theme.clone()
    }

    pub(crate) fn subscription(&self) -> Subscription<NyaaMessage> {
        match &self.current_view {
            NyaaView::QtorLibrary(_) => time::every(Duration::from_secs(2))
                .map(|_| NyaaMessage::Library(library::LibraryMessage::Load)),
            _ => Subscription::none(),
        }
    }
}
