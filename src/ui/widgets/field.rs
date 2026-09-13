use crate::appearance::{self, tokens};
use iced::Length::Fill;
use iced::alignment::Vertical::Center;
use iced::widget::{button, container, stack, text_input};
use iced::{Element, Padding, alignment, padding};
use iced_fonts::lucide::x;

pub(crate) struct Field<'a, Message> {
    placeholder: &'a str,
    value: &'a str,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
    on_clear: Option<Message>,
    enabled: bool,
}

impl<'a, Message> Field<'a, Message> {
    pub(crate) fn new(placeholder: &'a str, value: &'a str) -> Self {
        Self {
            placeholder,
            value,
            on_input: None,
            on_submit: None,
            on_clear: None,
            enabled: true,
        }
    }

    #[must_use]
    pub(crate) fn on_input(mut self, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    #[must_use]
    pub(crate) fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    #[must_use]
    pub(crate) fn on_clear(mut self, message: Message) -> Self {
        self.on_clear = Some(message);
        self
    }

    #[must_use]
    pub(crate) fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl<'a, Message: Clone + 'a> Field<'a, Message> {
    pub(crate) fn build(self) -> Element<'a, Message> {
        let Self {
            placeholder,
            value,
            on_input,
            on_submit,
            on_clear,
            enabled,
        } = self;

        let mut input = text_input(placeholder, value)
            .line_height(tokens::INPUT_LINE_HEIGHT)
            .size(tokens::INPUT_SIZE)
            .align_x(alignment::Horizontal::Center)
            .width(Fill)
            .style(appearance::text_input::primary);

        input = if on_clear.is_some() {
            input.padding(Padding::from(tokens::INPUT_PADDING).right(tokens::BUTTON_SIZE))
        } else {
            input.padding(tokens::INPUT_PADDING)
        };

        if enabled {
            if let Some(on_input) = on_input {
                input = input.on_input(on_input);
            }

            if let Some(on_submit) = on_submit {
                input = input.on_submit(on_submit);
            }
        }

        let clear_button = on_clear.map(|on_clear| {
            let clear_button = button(x().align_y(Center))
                .width(tokens::BUTTON_SIZE - 10)
                .height(tokens::BUTTON_SIZE - 10)
                .style(appearance::button::hover_only);

            if enabled {
                clear_button.on_press(on_clear)
            } else {
                clear_button
            }
        });

        let mut field = stack![input];

        if let Some(clear_button) = clear_button {
            field = field.push(
                container(clear_button)
                    .width(Fill)
                    .height(Fill)
                    .align_x(alignment::Horizontal::Right)
                    .align_y(Center)
                    .padding(padding::right(tokens::SPACING_SMALL)),
            );
        }

        field.width(Fill).into()
    }
}
