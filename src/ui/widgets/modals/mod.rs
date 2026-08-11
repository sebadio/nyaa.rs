use iced::{
    Color, Element, Task, Theme,
    widget::{center, container, mouse_area, opaque, text},
};
pub(crate) mod post_download;
pub(crate) mod save_settings;
use log::info;

pub(crate) enum Modal {
    PostDownload(post_download::Modal),
    SaveSettingsModal(save_settings::Modal),
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Cancel,
    PostDownload(post_download::Message),
    SaveSettings(save_settings::Message),
}

pub(crate) enum Event {
    CloseModal,
    PostModalCancel,
    PostDownloadSubmit(post_download::Options),
    SaveSettingsDecision(bool),
}

impl Modal {
    pub(crate) fn view(&self) -> Element<'_, Message> {
        match self {
            Self::PostDownload(modal) => modal.view().map(Message::PostDownload),
            Self::SaveSettingsModal(_modal) => text!("Next update?").into(),
        }
    }

    pub(crate) fn update(&mut self, message: Message) -> (iced::Task<Message>, Option<Event>) {
        match message {
            // I can match the cancel based on the current modal if we need to send a custom event on modal close
            // just need to add another arm and return the custom event
            Message::Cancel => match self {
                Self::PostDownload(_) => (Task::none(), Some(Event::PostModalCancel)),
                _ => (Task::none(), Some(Event::CloseModal)),
            },

            Message::PostDownload(message) => {
                let Modal::PostDownload(modal) = self else {
                    return (Task::none(), None);
                };

                match modal.update(message) {
                    post_download::Action::Cancel => {
                        info!("Canceled modal");
                        (Task::none(), Some(Event::PostModalCancel))
                    }
                    post_download::Action::None => (Task::none(), None),
                    post_download::Action::Submit(options) => {
                        (Task::none(), Some(Event::PostDownloadSubmit(options)))
                    }
                }
            }
        }
    }
}

pub(crate) fn modal<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    opaque(
        mouse_area(
            center(opaque(
                container(content)
                    .height(500)
                    .width(500)
                    .padding(24)
                    .style(modal_container_style),
            ))
            .style(modal_container_backdrop_style),
        )
        .on_press(on_blur),
    )
    .into()
}

fn modal_container_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        border: iced::Border {
            color: palette.background.strong.color,
            width: 2.0,
            radius: iced::border::radius(8),
        },
        background: Some(palette.background.base.color.into()),
        ..Default::default()
    }
}

fn modal_container_backdrop_style(theme: &Theme) -> container::Style {
    let black = theme.extended_palette().background.weakest.color;

    container::Style {
        background: Some(Color { a: 0.8, ..black }.into()),
        ..container::Style::default()
    }
}
