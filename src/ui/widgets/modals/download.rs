use iced::{
    Element,
    Length::Fill,
    widget::{button, column, row, space, text, toggler},
};

use nyaa::NyaaItem;

#[derive(Default, Debug, Clone)]
pub(crate) struct Options {
    pub(crate) open_on_finish: bool,
    pub(crate) add_to_library: bool,
}

#[derive(Clone)]
pub(crate) struct Modal {
    pub(crate) item: NyaaItem,
    options: Options,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    ToggleOpenOnFinish(bool),
    ToggleAddToLibrary(bool),
    Submit,
    Cancel,
}

pub(crate) enum Action {
    None,
    Cancel,
    Submit(Options),
}

impl Modal {
    pub(crate) fn new(item: NyaaItem) -> Self {
        Self {
            item,
            options: Options::default(),
        }
    }

    pub(crate) fn view<'a>(&self) -> Element<'a, Message> {
        column![
            row![text("What do we do after?").size(42)],
            space().height(12),
            row![
                text("Open on finish?").size(20).width(Fill),
                toggler(self.options.open_on_finish)
                    .on_toggle(Message::ToggleOpenOnFinish)
                    .size(32)
            ],
            row![
                text("Add to library?").size(20).width(Fill),
                toggler(self.options.add_to_library)
                    .on_toggle(Message::ToggleAddToLibrary)
                    .size(32)
            ],
            space().height(Fill),
            row![
                button(text("Cancel")).on_press(Message::Cancel),
                space().width(Fill),
                button("Submit").on_press(Message::Submit)
            ]
        ]
        .spacing(12)
        .into()
    }

    pub(crate) fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Cancel => return Action::Cancel,
            Message::Submit => return Action::Submit(self.options.clone()),
            Message::ToggleAddToLibrary(val) => self.options.add_to_library = val,
            Message::ToggleOpenOnFinish(val) => self.options.open_on_finish = val,
        }

        Action::None
    }
}
