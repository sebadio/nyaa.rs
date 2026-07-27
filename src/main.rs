mod qbittorrent;

use std::format;
use std::path::Path;

use chrono::{DateTime, Local, Utc};
use humansize::{DECIMAL, format_size};
use iced::Length::Fill;
use iced::time::{self, Duration};
use iced::widget::{
    button, column, container, mouse_area, row, scrollable, space, table, text, text_input,
};
use iced::window::Settings;
use iced::{Element, Font, Task, Theme, font, window};
use iced::{Size, Subscription};
use iced_fonts::lucide::{maximize, minus, x};
use iced_fonts::{
    LUCIDE_FONT_BYTES,
    lucide::{refresh_cw, settings},
};
use log::{info, warn};
use qbittorrent::{Client, Torrent};

#[derive(Default)]
enum NyaaView {
    #[default]
    NyaaSearch,
    QtorLibrary,
    Settings,
}

struct AppState {
    current_view: NyaaView,
    library_query: String,
    qbt_client: Client,
    library_torrents: Vec<Torrent>,
}

impl AppState {
    fn new() -> AppState {
        AppState {
            current_view: NyaaView::default(),
            library_query: String::new(),
            qbt_client: Client::new("http://127.0.0.1:8080").expect("client"),
            library_torrents: Vec::new(),
        }
    }

    fn view(app_state: &AppState) -> Element<'_, NyaaMessage> {
        column![
            // custom_titlebar(),
            column![row![sidebar(), column![main_view(app_state)].width(Fill)].spacing(12),]
                .padding(12)
        ]
        .into()
    }

    fn update(app_state: &mut AppState, message: NyaaMessage) -> Task<NyaaMessage> {
        match message {
            NyaaMessage::NavigateToSearch => app_state.current_view = NyaaView::NyaaSearch,
            NyaaMessage::NavigateToLibrary => {
                app_state.current_view = NyaaView::QtorLibrary;
                return Task::done(NyaaMessage::LoadTorrents);
            }
            NyaaMessage::NavigateToSettings => app_state.current_view = NyaaView::Settings,
            NyaaMessage::LibraryQueryChanged(text) => app_state.library_query = text,
            NyaaMessage::LoadTorrents => {
                let qbt = app_state.qbt_client.clone();
                return Task::perform(
                    async move {
                        qbt.login("iota", "cacatua123").await?;
                        qbt.get_torrents().await
                    },
                    NyaaMessage::TorrentsLoaded,
                );
            }
            NyaaMessage::TorrentsLoaded(Ok(list)) => app_state.library_torrents = list,
            NyaaMessage::TorrentsLoaded(Err(e)) => warn!("qbt error: {e:?}"),
            NyaaMessage::LibraryTorrentPressed(hash) => {
                info!("Pressed torrent: {}", hash);
                if let Some(t) = app_state.library_torrents.iter().find(|t| t.hash == hash) {
                    let _ = open::that(&t.content_path);
                }
            }
            NyaaMessage::Exit => return iced::exit(),
            NyaaMessage::ToggleWindowMode => {
                return window::latest().and_then(window::toggle_maximize);
            }
            NyaaMessage::Minimize => {
                return window::latest().and_then(|id| window::minimize(id, true));
            }
            NyaaMessage::Drag => return window::latest().and_then(window::drag),
            _ => warn!("Not Implemented yet!"),
        }

        Task::none()
    }
}

#[derive(Debug, Clone)]
enum NyaaMessage {
    Exit,
    ToggleWindowMode,
    Minimize,
    Drag,
    NavigateToSearch,
    NavigateToLibrary,
    NavigateToSettings,
    LibraryQueryChanged(String),
    LoadTorrents,
    TorrentsLoaded(Result<Vec<Torrent>, qbittorrent::Error>),
    LibraryTorrentPressed(String),
}

fn main() -> iced::Result {
    env_logger::init();

    let original_size = Size {
        width: 1200.0,
        height: 700.0,
    };

    iced::application(AppState::new, AppState::update, AppState::view)
        .default_font(Font::with_name("Monocraft"))
        // .decorations(false)
        // .default_font(Font::with_name("Miracode"))
        .font(LUCIDE_FONT_BYTES)
        .theme(iced::Theme::Ferra)
        .window(Settings {
            min_size: Some(original_size),
            size: original_size,
            ..Settings::default()
        })
        .subscription(subscription)
        .run()
}

fn subscription(app_state: &AppState) -> Subscription<NyaaMessage> {
    match app_state.current_view {
        NyaaView::QtorLibrary => {
            time::every(Duration::from_secs(1)).map(|_| NyaaMessage::LoadTorrents)
        }
        _ => Subscription::none(),
    }
}

fn custom_titlebar() -> Element<'static, NyaaMessage> {
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

fn sidebar() -> Element<'static, NyaaMessage> {
    column![
        row![text("Nyaa.rs").size(36)],
        space().height(20),
        button(text("Nyaa Search"))
            .on_press(NyaaMessage::NavigateToSearch)
            .width(Fill),
        button(text("Library"))
            .on_press(NyaaMessage::NavigateToLibrary)
            .width(Fill),
        space().height(Fill),
        button(settings())
            .on_press(NyaaMessage::NavigateToSettings)
            .width(Fill)
    ]
    .spacing(8)
    .width(200)
    .into()
}

fn library_table_button(torrent: Torrent) -> Element<'static, NyaaMessage> {
    let hash = torrent.hash.clone();
    button(text(torrent.name))
        .style(|theme: &Theme, status| {
            let palette = theme.extended_palette();
            button::Style {
                background: match status {
                    button::Status::Hovered => Some(palette.background.weak.color.into()),
                    _ => None,
                },
                text_color: match status {
                    button::Status::Hovered => palette.danger.strong.color,
                    _ => palette.background.base.text,
                },
                ..button::text(theme, status)
            }
        })
        .width(Fill)
        .on_press(NyaaMessage::LibraryTorrentPressed(hash))
        .into()
}

fn normalize(s: &str) -> String {
    s.to_lowercase().replace(['.', '_', '-'], " ")
}

fn is_video(name: &str) -> bool {
    const VIDEO_EXTS: [&str; 5] = ["mkv", "mp4", "avi", "mov", "webm"];
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| VIDEO_EXTS.iter().any(|v| e.eq_ignore_ascii_case(v)))
        .unwrap_or(false)
}

fn filter_library_torrents(query: &str, torrents: &[Torrent]) -> Vec<Torrent> {
    let normalized_query = normalize(query);
    let words: Vec<&str> = normalized_query.split_whitespace().collect();

    torrents
        .iter()
        .filter(|t| {
            let normalized_name = normalize(&t.name);
            is_video(&t.name) && words.iter().all(|w| normalized_name.contains(w))
        })
        .cloned()
        .collect()
}

fn library_view<'a>(query: &'a str, torrents: &[Torrent]) -> Element<'a, NyaaMessage> {
    let filtered_torrents = filter_library_torrents(&query, &torrents);

    let column_header_font = Font {
        weight: font::Weight::Bold,
        ..Font::default()
    };

    let columns = [
        table::column(text("Name").font(column_header_font), |t: Torrent| {
            library_table_button(t)
        })
        .width(Fill),
        table::column(text("Size").font(column_header_font), |t: Torrent| {
            text(format_size(t.size, DECIMAL))
        }),
        table::column(text("Seeders").font(column_header_font), |t: Torrent| {
            text(format!("{} ({})", t.num_seeds, t.num_complete))
        }),
        table::column(text("Leechs").font(column_header_font), |t: Torrent| {
            text(format!("{} ({})", t.num_leechs, t.num_incomplete))
        }),
        table::column(text("Progress").font(column_header_font), |t: Torrent| {
            text(format!("{:.0}%", t.progress * 100.0))
        }),
        table::column(text("Added On").font(column_header_font), |t: Torrent| {
            let dt_utc = DateTime::<Utc>::from_timestamp(t.added_on, 0).unwrap();
            let dt_local: DateTime<Local> = dt_utc.into();
            text(format!("{}", dt_local.format("%y-%m-%d %H:%M:%S")))
        }),
    ];

    let header_search = row![
        text_input("Search for downloaded torrents here", query)
            .on_input(NyaaMessage::LibraryQueryChanged)
            .padding(8)
            .width(Fill),
        space().width(12),
        button(refresh_cw().align_y(iced::alignment::Vertical::Center))
            .height(Fill)
            .on_press(NyaaMessage::LoadTorrents)
    ]
    .height(35)
    .align_y(iced::alignment::Vertical::Center);

    column![
        header_search,
        space().height(12),
        row![
            scrollable(table(columns, filtered_torrents.iter().cloned()))
                .spacing(5)
                .width(Fill)
        ]
        .height(Fill)
    ]
    .into()
}

fn main_view(app_state: &AppState) -> Element<'_, NyaaMessage> {
    match app_state.current_view {
        NyaaView::NyaaSearch => text("Nyaa let's search!").into(),
        NyaaView::QtorLibrary => {
            library_view(&app_state.library_query, &app_state.library_torrents)
        }
        NyaaView::Settings => text("Settings menu!").into(),
    }
}
