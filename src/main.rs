mod qbittorrent;

use humansize::{DECIMAL, format_size};
use iced::Length::Fill;
use iced::widget::{button, column, row, scrollable, space, table, text, text_input};
use iced::{Element, Task, Theme};
use log::{info, warn};
use qbittorrent::{Client, Torrent};

fn main() -> iced::Result {
    env_logger::init();
    iced::application(AppState::new, update, view)
        .theme(iced::Theme::Ferra)
        .run()
}

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
}

#[derive(Debug, Clone)]
enum NyaaMessage {
    Search,
    OpenDownload,
    NavigateToSearch,
    NavigateToLibrary,
    NavigateToSettings,
    LibraryQueryChanged(String),
    LoadTorrents,
    TorrentsLoaded(Result<Vec<Torrent>, qbittorrent::Error>),
    LibraryTorrentPressed(String),
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
        _ => warn!("Not Implemented yet!"),
    }

    Task::none()
}

fn view(app_state: &AppState) -> Element<'_, NyaaMessage> {
    column![row![sidebar(), column![main_view(app_state)].width(Fill)].spacing(12)]
        .padding(12)
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
        button(text("Settings"))
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

fn filter_library_torrents(query: &str, torrents: &[Torrent]) -> Vec<Torrent> {
    let formatted_query = query.to_lowercase();

    torrents
        .iter()
        .filter(|t| t.name.ends_with(".mkv"))
        .filter(|t| t.name.to_lowercase().contains(&formatted_query))
        .cloned()
        .collect()
}

fn library_view<'a>(query: &'a str, torrents: Vec<Torrent>) -> Element<'a, NyaaMessage> {
    let columns = [
        table::column(text("Name"), |t: Torrent| library_table_button(t)).width(Fill),
        table::column(text("Size"), |t: Torrent| {
            text(format_size(t.size, DECIMAL))
        }),
        table::column(text("Seeders"), |t: Torrent| text(t.num_seeds)),
        table::column(text("Leechs"), |t: Torrent| text(t.num_leechs)),
        table::column(text("Progress"), |t: Torrent| {
            text(format!("{:.0}%", t.progress * 100.0))
        }),
    ];

    let header_search = row![
        text_input("Search for downloaded torrents here", query)
            .on_input(NyaaMessage::LibraryQueryChanged)
            .width(Fill),
        space().width(12),
        button(text("Refresh")).on_press(NyaaMessage::LoadTorrents)
    ];

    column![
        header_search,
        space().height(12),
        row![
            scrollable(table(columns, torrents.iter().cloned()))
                .spacing(5)
                .width(Fill)
        ]
    ]
    .into()
}

fn main_view(app_state: &AppState) -> Element<'_, NyaaMessage> {
    match app_state.current_view {
        NyaaView::NyaaSearch => text("Nyaa let's search!").into(),
        NyaaView::QtorLibrary => library_view(
            &app_state.library_query,
            filter_library_torrents(&app_state.library_query, &app_state.library_torrents),
        ),
        NyaaView::Settings => text("Settings menu!").into(),
    }
}
