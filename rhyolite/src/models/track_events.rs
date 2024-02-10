use serde::Deserialize;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

use crate::models::{Severity, Track};

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Event {
    Start {
        guild_id: Id<GuildMarker>,
        track: Track,
    },
    End {
        guild_id: Id<GuildMarker>,
        track: Track,
        reason: TrackEndReason,
    },
    Exception {
        guild_id: Id<GuildMarker>,
        track: Track,
        exception: Severity,
    },
    Stuck {
        guild_id: Id<GuildMarker>,
        track: Track,
        threshold_ms: i32,
    },
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
