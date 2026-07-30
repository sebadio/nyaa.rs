use iced::Length::{Fill, FillPortion};
use iced::Theme;
use iced::widget::{button, column, pick_list, row, space, text, text_input};
use iced::{Alignment, Element, Task};
use iced_fonts::lucide::separator_horizontal;

use crate::config::Config;

pub(crate) struct Settings {
    pub(crate) config: Config,
}

#[derive(Debug, Clone)]
pub(crate) enum SettingsMessage {
    UpdatedConfig,
    ThemeChanged(Theme),
    UrlChanged(String),
    PassChanged(String),
    UsernameChanged(String),
}

pub(crate) enum Action {
    None,
    Run(Task<SettingsMessage>),
}

impl Settings {
    pub(crate) fn new(config: Config) -> Self {
        Self { config }
    }

    pub(crate) fn view<'a>(&self) -> Element<'_, SettingsMessage> {
        column![
            text("Config").size(48),
            separator_horizontal().width(Fill),
            row![
                text("Theme:")
                    .width(FillPortion(3))
                    .align_y(Alignment::Center),
                pick_list(
                    Theme::ALL,
                    self.config.theme.as_ref(),
                    SettingsMessage::ThemeChanged
                )
            ],
            row![
                text("qBittorrent URL:")
                    .width(FillPortion(3))
                    .align_y(Alignment::Center),
                text_input("127.0.0.1:8080", &self.config.qtor_url)
                    .width(FillPortion(1))
                    .on_input(SettingsMessage::UrlChanged)
            ],
            row![
                text("qBittorrent Username:")
                    .width(FillPortion(3))
                    .align_y(Alignment::Center),
                text_input("admin", &self.config.qtor_username)
                    .width(FillPortion(1))
                    .on_input(SettingsMessage::UsernameChanged)
            ],
            row![
                text("qBittorrent Password:")
                    .width(FillPortion(3))
                    .align_y(Alignment::Center),
                text_input("adminadmin", &self.config.qtor_pass)
                    .width(FillPortion(1))
                    .on_input(SettingsMessage::PassChanged)
            ],
            space().height(Fill),
            row![
                space().width(Fill),
                button(text("Apply changes")).on_press(SettingsMessage::UpdatedConfig)
            ]
        ]
        .spacing(10)
        .into()
    }

    pub fn update(&mut self, message: SettingsMessage, new_config: &mut Config) -> Action {
        match message {
            SettingsMessage::UrlChanged(url) => {
                self.config.qtor_url = url;

                Action::None
            }

            SettingsMessage::PassChanged(pass) => {
                self.config.qtor_pass = pass;

                Action::None
            }

            SettingsMessage::UsernameChanged(username) => {
                self.config.qtor_username = username;

                Action::None
            }

            SettingsMessage::ThemeChanged(theme) => {
                self.config.theme = Some(theme);

                Action::None
            }
            SettingsMessage::UpdatedConfig => {
                *new_config = self.config.clone();
                Action::None
            }
        }
    }
}
