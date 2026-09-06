use iced::{
    Element,
    Length::Fill,
    widget::{button, column, row, space, text, toggler},
};

use crate::appearance::tokens;

#[derive(Debug, Clone)]
pub(crate) struct Options {
    pub(crate) hash: String,
    pub(crate) delete_files: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct Modal {
    options: Options,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    ToggleDeleteFiles(bool),
    Submit,
    Cancel,
}

pub(crate) enum Action {
    None,
    Cancel,
    Submit(Options),
}

const TOGGLER_SIZE: u32 = 32;

impl Modal {
    pub(crate) fn new(hash: String) -> Self {
        Self {
            options: Options {
                hash,
                delete_files: true,
            },
        }
    }

    pub(crate) fn view<'a>(&self) -> Element<'a, Message> {
        column![
            text("Delete torrent?").size(tokens::TEXT_HEADER_SIZE),
            text("Are you sure you want to remove this torrent?"),
            row![
                text("Also remove downloaded files?").width(Fill),
                toggler(self.options.delete_files)
                    .on_toggle(Message::ToggleDeleteFiles)
                    .size(TOGGLER_SIZE)
            ]
            .spacing(tokens::SPACING_BASE),
            space().height(Fill),
            row![
                button(text("Cancel")).on_press(Message::Cancel),
                space().width(Fill),
                button(text("Delete")).on_press(Message::Submit)
            ]
        ]
        .spacing(tokens::SPACING_BASE)
        .into()
    }

    pub(crate) fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ToggleDeleteFiles(delete_files) => {
                self.options.delete_files = delete_files;
                Action::None
            }
            Message::Submit => Action::Submit(self.options.clone()),
            Message::Cancel => Action::Cancel,
        }
    }
}
