use serde::Deserialize;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

use crate::models::{Severity, Track};

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TrackEvents {
    TrackStartEvent {
        guild_id: Id<GuildMarker>,
        track: Track,
    },
    TrackEndEvent {
        guild_id: Id<GuildMarker>,
        track: Track,
        reason: TrackEndReason,
    },
    TrackExceptionEvent {
        guild_id: Id<GuildMarker>,
        track: Track,
        exception: Severity,
    },
    TrackStuckEvent {
        guild_id: Id<GuildMarker>,
        track: Track,
        threshold_ms: i32,
    },
    WebSocketClosedEvent {
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
