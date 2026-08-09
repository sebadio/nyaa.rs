use crate::nyaa_app::{ActiveDownload, NyaaMessage};
use crate::util::truncate_with_ellipsis;
use iced::widget::{progress_bar, row, text};
use iced::{
    Element,
    Length::{Fill, Fixed},
};

pub(crate) fn status_bar(download: &ActiveDownload) -> Element<'_, NyaaMessage> {
    let download_text = format!(
        "Downloading: {}",
        truncate_with_ellipsis(&download.name, 60)
    );

    row![
        text(download_text).width(Fill),
        progress_bar(0.0..=1.0, download.progress).length(Fixed(200.0)),
        text(format!("{:.0}%", download.progress * 100.0)),
    ]
    .padding(8)
    .height(40)
    .into()
}
