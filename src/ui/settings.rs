use iced::Length::FillPortion;
use iced::widget::{button, column, row, scrollable, space, table, text, text_input};
use iced::{Alignment, Element};

#[derive(Debug, Clone)]
pub(crate) enum SettingsMessage {
    UpdatedQtorUrl,
}

pub(crate) struct Settings {
    pub(crate) qtor_url: String,
    pub(crate) qtor_username: String,
    pub(crate) qtor_pass: String,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            qtor_url: String::new(),
            qtor_pass: String::new(),
            qtor_username: String::new(),
        }
    }

    pub(crate) fn view<'a>(&self) -> Element<'a, SettingsMessage> {
        column![row![
            text("qBittorrent URL:")
                .width(FillPortion(3))
                .align_y(Alignment::Center),
            text_input("127.0.0.1:8080", &self.qtor_url).width(FillPortion(1))
        ]]
        .into()
    }

    pub fn update(&self, message: SettingsMessage) {}
}
