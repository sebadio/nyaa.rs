use crate::nyaa_app::NyaaMessage;
use iced::Element;
use iced::Length::Fill;
use iced::widget::{button, container, mouse_area, row, space};
use iced_fonts::lucide::{maximize, minus, x};

pub(crate) fn titlebar() -> Element<'static, NyaaMessage> {
    mouse_area(
        container(
            row![
                space().width(Fill),
                button(minus()).on_press(NyaaMessage::Minimize),
                button(maximize()).on_press(NyaaMessage::ToggleWindowMode),
                button(x()).on_press(NyaaMessage::Exit),
            ]
            .width(Fill),
        )
        .height(32),
    )
    .on_press(NyaaMessage::Drag)
    .on_double_click(NyaaMessage::ToggleWindowMode)
    .into()
}
