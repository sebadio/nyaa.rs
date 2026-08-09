pub mod qbittorrent;
pub mod torrent;

pub use qbittorrent::{Client, Error};
pub use torrent::{Torrent, TorrentPostResponse};
