use iced::task::{Sipper, sipper};
use iced::time::Duration;
use log::error;
use qbittorrent::{self, Client, Error, Torrent};
use tokio::time::sleep;

pub(crate) fn track_torrent(
    client: &Client,
    hash: String,
) -> impl Sipper<Result<Torrent, qbittorrent::Error>, (String, f32)> + 'static {
    let client = client.clone();

    sipper(move |mut sender| async move {
        loop {
            match client.get_torrent_by_hash(&hash).await {
                Ok(t) if t.progress >= 1.0 => return Ok(t),
                Ok(t) => {
                    sender.send((t.name, t.progress)).await;
                }
                Err(e @ Error::TorrentNotFound(_)) => return Err(e),
                Err(e) => error!("{e}"),
            }

            sleep(Duration::from_secs(1)).await;
        }
    })
}
