use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Torrent {
    pub name: String,
    pub hash: String,
    pub content_path: String,
    pub size: u64,
    pub progress: f32,
    pub state: String,
    pub num_seeds: u32,
    pub num_leechs: u32,
    pub num_complete: u32,
    pub num_incomplete: u32,
    pub added_on: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TorrentPostResponse {
    pub added_torrent_ids: Vec<String>,
    pub failure_count: u32,
    pub pending_count: u32,
    pub success_count: u32,
}
