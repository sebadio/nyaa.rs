use crate::appearance::{self, tokens};
use crate::nyaa_app::{ActiveDownload, NyaaMessage};
use iced::alignment;
use iced::widget::{container, progress_bar, row, text};
use iced::{
    Element,
    Length::{Fill, Fixed},
};

pub(crate) fn status_bar(downloads: &[ActiveDownload]) -> Element<'static, NyaaMessage> {
    let content = match downloads {
        [] => row![text("Not downloading anything")],
        [download] => {
            let display_msg = if download.open_on_finish {
                "Waiting to open"
            } else {
                "Downloading"
            };
            let title = &download.name;

            row![
                text(format!("{display_msg}: {title}",))
                    .ellipsis(text::Ellipsis::End)
                    .wrapping(text::Wrapping::None)
                    .width(Fill),
                progress_bar(0.0..=1.0, download.progress)
                    .length(Fixed(200.0))
                    .girth(10)
                    .style(progress_bar::danger),
                text(format!("{:.0}%", download.progress * 100.0)),
            ]
        }

        downloads => {
            let average_progress = downloads
                .iter()
                .map(|download| download.progress)
                .sum::<f32>()
                / downloads.len() as f32;

            row![
                text(format!("Downloading {} torrents", downloads.len()))
                    .wrapping(text::Wrapping::None)
                    .width(Fill),
                progress_bar(0.0..=1.0, average_progress)
                    .length(Fixed(200.0))
                    .girth(10)
                    .style(progress_bar::danger),
                text(format!("{:.0}%", average_progress * 100.0)),
            ]
        }
    };

    container(
        content
            .align_y(alignment::Vertical::Center)
            .spacing(tokens::SPACING_SMALL)
            .width(Fill)
            .padding(tokens::STATUS_BAR_PADDING),
    )
    .style(appearance::container::weakest)
    .height(tokens::STATUS_BAR_HEIGHT)
    .into()
}
