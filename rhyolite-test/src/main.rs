use std::sync::Arc;
use std::time::Duration;

use twilight_gateway::{Intents, Shard, ShardId};
use twilight_model::gateway::payload::outgoing::UpdateVoiceState;
use twilight_model::id::marker::{ChannelMarker, GuildMarker};
use twilight_model::id::Id;

use rhyolite::models::TrackLoadingResult;
use rhyolite::{RhyoliteCache, RhyoliteHttp, RhyoliteWsClient};

#[tokio::main]
async fn main() {
    let token = std::env::var("BOT_TOKEN").unwrap();

    let mut cache = Arc::new(RhyoliteCache::default());
    let http = RhyoliteHttp::new("localhost:2333", "abc");

    let twil_http = twilight_http::Client::new(token.clone());
    let user_id = twil_http
        .current_user()
        .await
        .unwrap()
        .model()
        .await
        .unwrap()
        .id;
    println!("User id is {}", user_id);

    let request = http.load_tracks("ytsearch:INTERNET YAMERO").await.unwrap();
    if let TrackLoadingResult::Search(s) = request {
        for track in s.iter().take(5) {
            println!(
                "Found track \"{}\" from author {}",
                track.info.title, track.info.author
            );
        }
    }

    let request = http.load_tracks("BidG5Goe4RU").await.unwrap();
    if let TrackLoadingResult::Track(t) = request {
        println!(
            "Got track \"{}\" from author {}",
            t.info.title, t.info.author
        );
    }

    let mut ws_client =
        RhyoliteWsClient::new("localhost:2333", "abc", &user_id.to_string(), cache.clone()).await;

    let task = tokio::spawn(async move {
        loop {
            println!("Getting lavalink event");
            let result = ws_client.next().await;
            if result.is_err() {
                break;
            }
            println!("Lavalink event: {:?}", result);
        }
    });

    let intents = Intents::GUILD_MESSAGES | Intents::GUILD_VOICE_STATES;
    let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let guild_id: Id<GuildMarker> = Id::new(654714396052029451);
    let channel_id: Id<ChannelMarker> = Id::new(862451568044277780);
    shard
        .sender()
        .command(&UpdateVoiceState::new(guild_id, channel_id, true, false))
        .unwrap();

    loop {
        let event = match shard.next_event().await {
            Ok(e) => e,
            Err(err) => {
                println!("Errored with: {}", err);
                continue;
            }
        };

        let result = cache.process_vc_event(event);
        if let Err(e) = result {
            println!("Errored with: {}", e);
        }
    }
}
