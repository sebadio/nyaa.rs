use iced::widget::{center, container, mouse_area, opaque, text};
use iced::{Element, Task};
pub(crate) mod delete_torrent;
pub(crate) mod download;
pub(crate) mod save_settings;
use log::info;

use crate::appearance::{self, tokens};

pub(crate) enum Modal {
    DeleteTorrent(delete_torrent::Modal),
    Download(download::Modal),
    SaveSettingsModal(save_settings::Modal),
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Cancel,
    DeleteTorrent(delete_torrent::Message),
    PostDownload(download::Message),
    SaveSettings(save_settings::Message),
}

pub(crate) enum Event {
    CloseModal,
    PostDeleteTorrentSubmit(delete_torrent::Options),
    PostModalCancel,
    PostDownloadSubmit(download::Options),
    SaveSettingsDecision(bool),
}

impl Modal {
    pub(crate) fn view(&self) -> Element<'_, Message> {
        match self {
            Self::DeleteTorrent(modal) => modal.view().map(Message::DeleteTorrent),
            Self::Download(modal) => modal.view().map(Message::PostDownload),
            Self::SaveSettingsModal(_modal) => text!("Next update?").into(),
        }
    }

    pub(crate) fn update(&mut self, message: Message) -> (iced::Task<Message>, Option<Event>) {
        match message {
            // I can match the cancel based on the current modal if we need to send a custom event on modal close
            // just need to add another arm and return the custom event
            Message::Cancel => match self {
                Self::Download(_) => (Task::none(), Some(Event::PostModalCancel)),
                _ => (Task::none(), Some(Event::CloseModal)),
            },

            Message::DeleteTorrent(message) => {
                let Modal::DeleteTorrent(modal) = self else {
                    return (Task::none(), None);
                };

                match modal.update(message) {
                    delete_torrent::Action::Cancel => (Task::none(), Some(Event::CloseModal)),
                    delete_torrent::Action::None => (Task::none(), None),
                    delete_torrent::Action::Submit(options) => {
                        (Task::none(), Some(Event::PostDeleteTorrentSubmit(options)))
                    }
                }
            }

            Message::PostDownload(message) => {
                let Modal::Download(modal) = self else {
                    return (Task::none(), None);
                };

                match modal.update(message) {
                    download::Action::Cancel => {
                        info!("Canceled modal");
                        (Task::none(), Some(Event::PostModalCancel))
                    }
                    download::Action::None => (Task::none(), None),
                    download::Action::Submit(options) => {
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
                    .padding(tokens::MODAL_PADDING)
                    .style(appearance::container::modal),
            ))
            .style(appearance::container::backdrop),
        )
        .on_press(on_blur),
    )
    .into()
}
