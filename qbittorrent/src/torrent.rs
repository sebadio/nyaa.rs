use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum State {
    #[serde(rename = "error")]
    Error,

    #[serde(rename = "missingFiles")]
    MissingFiles,

    #[serde(rename = "uploading")]
    Uploading,

    #[serde(rename = "pausedUP", alias = "stoppedUP")]
    PausedUp,

    #[serde(rename = "queuedUP")]
    QueuedUp,

    #[serde(rename = "stalledUP")]
    StalledUp,

    #[serde(rename = "checkingUP")]
    CheckingUp,

    #[serde(rename = "forcedUP")]
    ForcedUp,

    #[serde(rename = "allocating")]
    Allocating,

    #[serde(rename = "downloading")]
    Downloading,

    #[serde(rename = "metaDL")]
    MetaDl,

    #[serde(rename = "pausedDL", alias = "stoppedDL")]
    PausedDl,

    #[serde(rename = "queuedDL")]
    QueuedDl,

    #[serde(rename = "stalledDL")]
    StalledDl,

    #[serde(rename = "checkingDL")]
    CheckingDl,

    #[serde(rename = "forceDL")]
    ForceDl,

    #[serde(rename = "checkingResumeData")]
    CheckingResumeData,

    #[serde(rename = "moving")]
    Moving,

    #[serde(rename = "unknown")]
    Unknown,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            State::Error => "Error",
            State::MissingFiles => "Missing Files",

            State::Uploading | State::StalledUp | State::ForcedUp => "Seeding",

            State::PausedUp => "Completed",

            State::QueuedUp | State::QueuedDl => "Queued",

            State::CheckingUp | State::CheckingDl | State::CheckingResumeData => "Checking",

            State::Allocating => "Allocating",

            State::Downloading | State::ForceDl => "Downloading",

            State::MetaDl => "Downloading Metadata",

            State::PausedDl => "Stopped",

            State::StalledDl => "Stalled",

            State::Moving => "Moving",

            State::Unknown => "Unknown",
        };

        f.write_str(name)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Torrent {
    pub added_on: i64,
    pub amount_left: u64,
    pub auto_tmm: bool,
    pub availability: f64,

    pub category: String,
    pub comment: String,

    pub completed: u64,
    pub completion_on: i64,

    pub connections_count: u32,
    pub connections_limit: i64,

    pub content_path: String,
    pub created_by: String,
    pub creation_date: i64,

    pub dl_limit: i64,
    pub dlspeed: u64,
    pub download_path: String,
    pub downloaded: u64,
    pub downloaded_session: u64,

    pub eta: i64,

    pub f_l_piece_prio: bool,
    pub force_start: bool,
    pub has_metadata: bool,

    pub hash: String,
    pub infohash_v1: String,
    pub infohash_v2: String,

    pub inactive_seeding_time_limit: i64,

    pub last_activity: i64,

    pub magnet_uri: String,

    pub max_inactive_seeding_time: i64,
    pub max_ratio: f64,
    pub max_seeding_time: i64,

    pub name: String,

    pub num_complete: i64,
    pub num_incomplete: i64,
    pub num_leechs: u32,
    pub num_seeds: u32,

    pub piece_size: u64,
    pub pieces_have: u64,
    pub pieces_num: u64,

    pub popularity: f64,
    pub priority: i64,
    pub private: bool,
    pub progress: f32,

    pub ratio: f64,
    pub ratio_limit: f64,
    pub reannounce: i64,

    pub root_path: String,
    pub save_path: String,

    pub seeding_time: i64,
    pub seeding_time_limit: i64,
    pub seen_complete: i64,

    pub seq_dl: bool,
    pub share_limit_action: String,

    pub size: u64,
    pub state: State,

    pub super_seeding: bool,

    pub tags: String,

    pub time_active: i64,
    pub total_size: u64,
    pub total_wasted: u64,

    pub tracker: String,
    pub trackers_count: u32,

    pub up_limit: i64,
    pub uploaded: u64,
    pub uploaded_session: u64,
    pub upspeed: u64,
}

impl Torrent {
    pub fn progress_percent(&self) -> f32 {
        self.progress * 100.0
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }

    pub fn is_downloading(&self) -> bool {
        matches!(
            self.state,
            State::Downloading | State::ForceDl | State::StalledDl | State::QueuedDl
        )
    }

    pub fn is_seeding(&self) -> bool {
        matches!(
            self.state,
            State::Uploading | State::ForcedUp | State::StalledUp | State::QueuedUp
        )
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TorrentPostResponse {
    pub added_torrent_ids: Vec<String>,
    pub failure_count: u32,
    pub pending_count: u32,
    pub success_count: u32,
}
