use crate::config::Config;
use iced::Length::{Fill, FillPortion};
use iced::border::Radius;
use iced::widget::{Text, button, column, pick_list, row, rule, space, text, text_input};
use iced::{Alignment, Element, Task};
use iced::{Color, Theme};
use iced_fonts::lucide::folder;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct Settings {
    pub(crate) config: Config,
    is_path_valid: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum SettingsMessage {
    UpdatedConfig,
    ThemeChanged(Theme),
    UrlChanged(String),
    PassChanged(String),
    UsernameChanged(String),
    SavePathChanged(String),
    PickSavePath,
    SavePathPicked(Option<PathBuf>),
}

pub(crate) enum Action {
    None,
    Run(Task<SettingsMessage>),
}

impl Settings {
    pub(crate) fn new(config: Config) -> Self {
        Self {
            config,
            is_path_valid: true,
        }
    }

    pub(crate) fn view(&self) -> Element<'_, SettingsMessage> {
        let validty_message: Text<'_, iced::Theme> = if self.is_path_valid {
            text("Valid path").color(Color::from_rgba(0.0, 1.0, 0.0, 1.0))
        } else {
            text("Invalid Path").color(Color::from_rgba(1.0, 0.0, 0.0, 1.0))
        };

        column![
            text("Config").size(48),
            rule::horizontal(2).style(|theme: &Theme| rule::Style {
                fill_mode: rule::FillMode::Full,
                radius: Radius::default(),
                snap: true,
                color: theme.extended_palette().secondary.weak.color
            }),
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
            row![
                text("qBittorrent Save Path:")
                    .width(FillPortion(3))
                    .align_y(Alignment::Center),
                column![
                    row![
                        text_input("~/Downloads", &self.config.qtor_save_path)
                            .width(FillPortion(1))
                            .on_input(SettingsMessage::SavePathChanged),
                        space().width(2),
                        button(folder()).on_press(SettingsMessage::PickSavePath)
                    ],
                    validty_message.size(10)
                ]
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

            SettingsMessage::SavePathChanged(path) => {
                self.config.qtor_save_path = path.clone();
                self.is_path_valid = is_writable_dir(&path);

                Action::None
            }

            SettingsMessage::PickSavePath => {
                let start = self.config.qtor_save_path.clone();
                Action::Run(Task::perform(
                    pick_folder(start),
                    SettingsMessage::SavePathPicked,
                ))
            }

            SettingsMessage::SavePathPicked(None) => Action::None,
            SettingsMessage::SavePathPicked(Some(path)) => {
                self.config.qtor_save_path = path.display().to_string();
                self.is_path_valid = is_writable_dir(&self.config.qtor_save_path);

                Action::None
            }

            SettingsMessage::UpdatedConfig => {
                *new_config = self.config.clone();
                Action::None
            }
        }
    }
}

async fn pick_folder(start: String) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Choose save location")
        .set_directory(if start.is_empty() { "~".into() } else { start })
        .pick_folder()
        .await
        .map(|h| h.path().to_path_buf())
}

fn is_writable_dir<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    if !path.is_dir() {
        return false;
    }

    let probe = path.join(format!(".write_test_{}", std::process::id()));
    match fs::File::create(&probe) {
        Ok(_) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}
