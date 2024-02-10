use serde::Deserialize;
use twilight_model::guild::Guild;
use twilight_model::id::Id;

pub mod request;
pub mod track_events;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Ready {
    pub resumed: bool,
    pub session_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayerUpdate {
    pub guild_id: Id<Guild>,
    pub state: PlayerState,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayerState {
    pub time: i32,
    pub position: i32,
    pub connected: bool,
    pub ping: i32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub players: i32,
    pub playing_players: i32,
    pub uptime: i32,
    pub memory: Memory,
    pub cpu: Cpu,
    pub frame_stats: Option<FrameStats>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    pub free: i64,
    pub used: i64,
    pub allocated: i64,
    pub reservable: i64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Cpu {
    pub cores: i32,
    pub system_load: f32,
    pub lavalink_load: f32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FrameStats {
    pub sent: i32,
    pub nulled: i32,
    /// ## The difference between sent frames and the expected amount of frames
    /// The expected amount of frames is 3000 (1 every 20 ms) per player. If the deficit is negative, too many frames were sent, and if it's positive, not enough frames got sent.
    pub deficit: i32,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub encoded: String,
    pub info: TrackInfo,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    pub identifier: String,
    pub is_seekable: bool,
    pub author: String,
    pub length: i32,
    pub is_stream: bool,
    pub position: i32,
    pub title: String,
    pub uri: Option<String>,
    pub artwork_url: Option<String>,
    pub isrc: Option<String>,
    pub source_name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "loadType", content = "data")]
pub enum LoadTracksResult {
    Track(Track),
    Playlist(Playlist),
    Search(Vec<Track>),
    Empty,
    Error(LavalinkException),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistInfo {
    pub name: String,
    pub selected_track: i32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub info: PlaylistInfo,
    pub tracks: Vec<Track>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LavalinkException {
    pub message: Option<String>,
    pub severity: Severity,
    pub cause: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Common,
    Suspicious,
    Fault,
}
