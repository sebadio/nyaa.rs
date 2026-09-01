use crate::appearance;
use crate::appearance::tokens::{
    BTN_PADDING, BUTTON_SIZE, PADDING_NONE, PANEL_PADDING, RADIUS_LARGE, SPACING_BASE,
    SPACING_SMALL, SPACING_TINY, TEXT_SMALL,
};
use crate::ui::downloads::DownloadsMessage;
use humansize::{BINARY, format_size};
use humantime::format_duration;
use iced::Length::Fill;
use iced::border::Radius;
use iced::widget::{button, center, column, container, progress_bar, row, space, text};
use iced::{Border, Color, Element, Theme, alignment};
use iced_fonts::lucide::{arrow_down, arrow_up, external_link, octagon_alert, pause, play, trash};
use qbittorrent::{State, Torrent};
use std::time::Duration;

pub(crate) fn download_item(torrent: &Torrent) -> Element<'static, DownloadsMessage> {
    let download = format!("{}/s", format_size(torrent.dlspeed, BINARY));
    let upload = format!("{}/s", format_size(torrent.upspeed, BINARY));

    let (button_icon, button_action) = match torrent.state {
        State::PausedDl | State::PausedUp => (
            play(),
            Some(DownloadsMessage::ResumeTorrent(torrent.hash.clone())),
        ),

        State::Unknown | State::Error | State::MissingFiles => (octagon_alert(), None),
        _ => (
            pause(),
            Some(DownloadsMessage::PauseTorrent(torrent.hash.clone())),
        ),
    };

    let open_action = match torrent.is_complete() {
        true => Some(DownloadsMessage::TorrentPressed(torrent.hash.clone())),
        false => None,
    };

    let eta_text = match torrent.is_downloading() {
        true => text(format!(
            "ETA: {}",
            format_duration(Duration::from_secs(torrent.eta as u64))
        )),
        false => text(format!("{}", torrent.state)),
    };

    let dlup_stats: Element<'static, DownloadsMessage> = match torrent.state {
        State::PausedUp => space().into(),
        _ => row![
            row![
                arrow_up().style(appearance::text::seeder).size(TEXT_SMALL),
                text(upload)
                    .style(appearance::text::seeder)
                    .size(TEXT_SMALL)
            ]
            .spacing(SPACING_TINY)
            .align_y(alignment::Vertical::Center),
            row![
                arrow_down()
                    .style(appearance::text::leecher)
                    .size(TEXT_SMALL),
                text(download)
                    .style(appearance::text::leecher)
                    .size(TEXT_SMALL)
            ]
            .spacing(SPACING_TINY)
            .align_y(alignment::Vertical::Center),
        ]
        .into(),
    };

    container(
        column![
            row![
                button(center(button_icon))
                    .on_press_maybe(button_action)
                    .width(BUTTON_SIZE)
                    .height(BUTTON_SIZE)
                    .padding(BTN_PADDING)
                    .style(appearance::button::secondary_action),
                column![
                    button(
                        text(torrent.name.clone())
                            .ellipsis(text::Ellipsis::End)
                            .wrapping(text::Wrapping::None)
                    )
                    .padding(PADDING_NONE)
                    .style(appearance::button::title_link)
                    .on_press_maybe(open_action.clone()),
                    row![
                        text(format!("{}", torrent.state.clone()))
                            .size(TEXT_SMALL)
                            .style(text_color(torrent.state.clone())),
                        dlup_stats,
                        space().width(Fill),
                        eta_text.size(TEXT_SMALL),
                    ]
                    .spacing(SPACING_SMALL)
                ]
                .spacing(SPACING_SMALL)
                .width(Fill),
                button(center(external_link()))
                    .on_press_maybe(open_action)
                    .width(BUTTON_SIZE)
                    .height(BUTTON_SIZE)
                    .padding(BTN_PADDING)
                    .style(appearance::button::secondary_action),
                button(center(trash()))
                    .on_press(DownloadsMessage::RemoveTorrent(torrent.hash.clone()))
                    .width(BUTTON_SIZE)
                    .height(BUTTON_SIZE)
                    .padding(BTN_PADDING)
                    .style(appearance::button::secondary_action)
            ]
            .spacing(SPACING_SMALL),
            row![
                progress_bar(0.0..=1.0, torrent.progress)
                    .girth(SPACING_SMALL)
                    .style(progress_color(torrent.state.clone())),
                text(format!("{:.0}%", torrent.progress * 100.0))
            ]
            .align_y(alignment::Vertical::Center)
            .spacing(SPACING_BASE)
        ]
        .width(Fill)
        .spacing(SPACING_TINY),
    )
    .width(Fill)
    .padding(PANEL_PADDING)
    .style(appearance::container::base)
    .into()
}

fn text_color(state: State) -> impl Fn(&Theme) -> text::Style {
    move |theme: &Theme| text::Style {
        color: Some(highlight(theme, state)),
    }
}

fn progress_color(state: State) -> impl Fn(&Theme) -> progress_bar::Style {
    move |theme: &Theme| progress_bar::Style {
        border: Border {
            radius: Radius::from(RADIUS_LARGE),
            ..Default::default()
        },
        bar: iced::Background::Color(highlight(theme, state)),
        ..progress_bar::secondary(theme)
    }
}

fn highlight(theme: &Theme, state: State) -> Color {
    let palette = theme.palette();

    match state {
        State::Error | State::MissingFiles => palette.danger.base.color,
        State::Downloading | State::ForceDl | State::MetaDl | State::Allocating => {
            palette.success.base.color
        }
        State::Uploading | State::StalledUp | State::ForcedUp => palette.primary.strong.color,
        State::PausedUp => palette.secondary.weak.color,
        State::StalledDl => palette.warning.base.color,
        State::PausedDl => palette.secondary.weak.color,
        State::QueuedUp
        | State::QueuedDl
        | State::CheckingUp
        | State::CheckingDl
        | State::CheckingResumeData
        | State::Moving => palette.secondary.weak.color,
        State::Unknown => palette.secondary.base.color,
    }
}
