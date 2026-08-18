use crate::nyaa_app::{ActiveDownload, NyaaMessage};
use iced::alignment;
use iced::widget::{Theme, container, progress_bar, row, text};
use iced::{
    Element,
    Length::{Fill, Fixed},
};

pub(crate) fn status_bar(active_download: Option<ActiveDownload>) -> Element<'static, NyaaMessage> {
    let content = match active_download {
        Some(active_download) => {
            let display_msg = if active_download.open_on_finish {
                "Waiting to open"
            } else {
                "Downloading"
            };
            let title = &active_download.name;

            row![
                text(format!("{display_msg}: {title}",))
                    .ellipsis(text::Ellipsis::End)
                    .wrapping(text::Wrapping::None)
                    .width(Fill),
                progress_bar(0.0..=1.0, active_download.progress)
                    .length(Fixed(200.0))
                    .girth(10)
                    .style(progress_bar::danger),
                text(format!("{:.0}%", active_download.progress * 100.0)),
            ]
        }
        None => row![text("Not downloading anything")],
    };

    container(
        content
            .align_y(alignment::Vertical::Center)
            .spacing(8)
            .width(Fill)
            .padding([0.0, 8.0]),
    )
    .style(|theme: &Theme| container::Style {
        background: Some(theme.palette().background.weakest.color.into()),
        ..Default::default()
    })
    .height(20)
    .into()
}
