use serde::Deserialize;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

use crate::models::{Severity, Track};

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "TrackStartEvent")]
    Start {
        guild_id: Id<GuildMarker>,
        track: Track,
    },
    #[serde(rename = "TrackEndEvent")]
    End {
        guild_id: Id<GuildMarker>,
        track: Track,
        reason: TrackEndReason,
    },
    #[serde(rename = "TrackExceptionEvent")]
    Exception {
        guild_id: Id<GuildMarker>,
        track: Track,
        exception: Severity,
    },
    #[serde(rename = "TrackStuckEvent")]
    Stuck {
        guild_id: Id<GuildMarker>,
        track: Track,
        threshold_ms: i32,
    },
    #[serde(rename = "WebSocketClosedEvent")]
    WebSocketClosed {
        guild_id: Id<GuildMarker>,
        code: i32,
        reason: String,
        by_remote: bool,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TrackEndReason {
    Finished,
    LoadFailed,
    Stopped,
    Replaced,
    Cleanup,
}
