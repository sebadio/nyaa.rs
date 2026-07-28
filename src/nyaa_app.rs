use crate::qbittorrent::Client;
use crate::ui::settings::{self, Settings};
use crate::ui::{Library, library};
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
}

impl NyaaAppState {
    pub(crate) fn new() -> NyaaAppState {
        NyaaAppState {
            current_view: NyaaView::default(),
            qbt_client: Client::new("http://127.0.0.1:8080").expect("client"),
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
                self.current_view = NyaaView::QtorLibrary(Library::new());
                return Task::done(NyaaMessage::Library(library::LibraryMessage::Load));
            }
            NyaaMessage::NavigateToSettings => {
                self.current_view = NyaaView::Settings(Settings::new());
                return Task::none();
            }
            NyaaMessage::Settings(settings_message) => {
                let NyaaView::Settings(settings) = &mut self.current_view else {
                    return Task::none();
                };

                return match settings.update(settings_message) {
                    _ => todo!("TODO"),
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

    pub(crate) fn subscription(&self) -> Subscription<NyaaMessage> {
        match &self.current_view {
            NyaaView::QtorLibrary(_) => time::every(Duration::from_secs(1))
                .map(|_| NyaaMessage::Library(library::LibraryMessage::Load)),
            _ => Subscription::none(),
        }
    }
}
