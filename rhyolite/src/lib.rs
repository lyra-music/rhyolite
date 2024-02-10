use dashmap::DashMap;
use futures_util::stream::FusedStream;
use futures_util::StreamExt;
use reqwest::header::HeaderMap;
use reqwest::ClientBuilder;
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{tungstenite::Error as TungsteniteError, MaybeTlsStream, WebSocketStream};
use twilight_model::gateway::event::Event as TwilightEvent;
use twilight_model::gateway::payload::incoming::{VoiceServerUpdate, VoiceStateUpdate};
use twilight_model::id::marker::{ChannelMarker, GuildMarker};
use twilight_model::id::Id;
use url::Url;

use crate::models::track_events::Event as TrackEvents;
use crate::models::{LoadTracksResult, Track};

pub mod models;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Connection was closed")]
    ConnectionClosed,
    #[error("Invalid OP type (should never happen)")]
    InvalidOp,
    #[error("Can't reconnect to the ws, either cause no session id was found or connection did not accept")]
    CouldntReconnect,
    #[error("Cannot reconnect when already connected")]
    AlreadyConnected,
    #[error("Couldn't send HTTP request")]
    HttpRequestFailed,
}

pub struct WebSocketClient {
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    connection_url: String,
    authorization: String,
    bot_id: String,
    session_id: Option<String>,
    cache: Arc<InMemoryCache>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Event {
    Ready(models::Ready),
    PlayerUpdate(models::PlayerUpdate),
    Stats(models::Stats),
    Track(TrackEvents),
}

impl WebSocketClient {
    pub async fn new(
        connection_url: &str,
        authorization: &str,
        bot_id: &str,
        cache: Arc<InMemoryCache>,
    ) -> Self {
        let url =
            Url::parse(&format!("ws://{}/v4/websocket", connection_url)).expect("Invalid url");
        let mut req = url.into_client_request().unwrap();

        let headers = req.headers_mut();
        headers.insert("Authorization", authorization.parse().unwrap());
        headers.insert("User-Id", bot_id.parse().unwrap());
        headers.insert(
            "Client-Name",
            format!("RHYOLITE/{}", env!("CARGO_PKG_VERSION"))
                .parse()
                .unwrap(),
        );

        let (ws_stream, _) = tokio_tungstenite::connect_async(req)
            .await
            .expect("Cannot connect to target");

        let mut client = Self {
            connection_url: connection_url.to_string(),
            ws_stream,
            authorization: authorization.to_string(),
            bot_id: bot_id.to_string(),
            session_id: None,
            cache,
        };
        let event = client.next_event().await;
        if let Ok(Event::Ready(r)) = event {
            client.session_id = Some(r.session_id);
        } else {
            panic!("First event wasn't ready.");
        }

        client
    }

    pub async fn next_event(&mut self) -> Result<Event, Error> {
        let msg = loop {
            let msg = match self.ws_stream.next().await {
                Some(Ok(m)) => m,
                Some(Err(e)) => {
                    match e {
                        TungsteniteError::ConnectionClosed => {
                            return Err(Error::ConnectionClosed);
                        } // To not continue an infinite loop of reading
                        TungsteniteError::AlreadyClosed => return Err(Error::ConnectionClosed), // Same reason as above
                        _ => continue, // Bad and evil
                    };
                }
                None => continue,
            };

            if msg.is_text() {
                break msg.into_text().unwrap();
            }
        };

        let event = serde_json::from_str::<Event>(&msg).map_err(|_| Error::InvalidOp);
        if let Ok(Event::Track(e)) = event.as_ref() {
            self.update_player(e);
        }

        event
    }

    pub async fn disconnect(mut self) {
        let err = self.ws_stream.close(None).await;
        if let Err(e) = err {
            panic!("Couldn't disconnect without error: {}", e);
        }
    }

    pub async fn reconnect(&mut self) -> Result<(), Error> {
        if !self.ws_stream.is_terminated() {
            return Err(Error::AlreadyConnected);
        }

        let url =
            Url::parse(&format!("ws://{}/v4/websocket", self.connection_url)).expect("Invalid url");
        let mut req = url.into_client_request().unwrap();

        let headers = req.headers_mut();
        headers.insert("Authorization", self.authorization.parse().unwrap());
        headers.insert("User-Id", self.bot_id.parse().unwrap());
        headers.insert(
            "Client-Name",
            format!("RHYOLITE/{}", env!("CARGO_PKG_VERSION"))
                .parse()
                .unwrap(),
        );
        if let Some(s) = self.session_id.as_ref() {
            headers.insert("Session-Id", s.parse().unwrap());
        } else {
            return Err(Error::CouldntReconnect);
        }

        let ws_connect = tokio_tungstenite::connect_async(req).await;
        if ws_connect.is_err() {
            return Err(Error::CouldntReconnect);
        }
        let (ws_stream, _) = ws_connect.unwrap();
        self.ws_stream = ws_stream;

        let event = self.next_event().await;
        if let Ok(Event::Ready(r)) = event {
            self.session_id = Some(r.session_id);
        } else {
            panic!("First event wasn't ready.");
        }

        Ok(())
    }

    fn update_player(&mut self, track_event: &TrackEvents) {
        match track_event {
            TrackEvents::Start { guild_id, track } => {
                let mut player = self
                    .cache
                    .players
                    .get_mut(guild_id)
                    .expect("Player was never cached");
                player.track = Some(track.clone());
            }
            TrackEvents::End { guild_id, .. } => {
                let mut player = self
                    .cache
                    .players
                    .get_mut(guild_id)
                    .expect("Player was never cached");
                player.track = None;
            }
            TrackEvents::WebSocketClosed { guild_id, .. } => {
                self.cache.players.remove(guild_id);
            }
            _ => {}
        };
    }
}

pub struct InMemoryCache {
    guild_tokens: DashMap<Id<GuildMarker>, GuildToken>,
    players: DashMap<Id<GuildMarker>, Player>,
}

struct GuildToken {
    token: String,
    endpoint: Option<String>,
    channel_sessions: DashMap<Id<ChannelMarker>, String>,
}

pub struct Player {
    pub channel_id: Id<ChannelMarker>,
    pub track: Option<Track>,
}

impl InMemoryCache {
    pub fn process_voice_event(&self, event: TwilightEvent) -> Result<(), Error> {
        if let TwilightEvent::VoiceStateUpdate(v) = event {
            println!("Processing voice state update.");
            return self.handle_voice_state_update(v);
        } else if let TwilightEvent::VoiceServerUpdate(v) = event {
            println!("Processing voice server update.");
            return self.handle_voice_server_update(v);
        }

        Ok(())
    }

    fn handle_voice_server_update(&self, e: VoiceServerUpdate) -> Result<(), Error> {
        let player = self.players.get_mut(&e.guild_id);
        if let Some(mut _p) = player {
            // TODO: Update the player
        }

        // The cache should always be up-to-date if user wants to change channel
        let guild = self.guild_tokens.get_mut(&e.guild_id);

        if let Some(mut g) = guild {
            g.token = e.token;
            g.endpoint = e.endpoint;
        } else {
            let guild_token = GuildToken {
                channel_sessions: DashMap::new(),
                endpoint: e.endpoint,
                token: e.token,
            };
            self.guild_tokens.insert(e.guild_id, guild_token);
        }

        Ok(())
    }

    fn handle_voice_state_update(&self, e: Box<VoiceStateUpdate>) -> Result<(), Error> {
        if let (Some(g), Some(c)) = (e.guild_id, e.channel_id) {
            let player = self.players.get_mut(&g);
            if let Some(p) = player {
                if p.channel_id != c {
                    return Ok(());
                }
                // TODO: Update the player
            }

            let guild = self.guild_tokens.get_mut(&g);
            if let Some(g) = guild {
                g.channel_sessions.insert(c, e.session_id.clone());
            }
        }

        Ok(())
    }
}

impl Default for InMemoryCache {
    fn default() -> Self {
        InMemoryCache {
            guild_tokens: DashMap::new(),
            players: DashMap::new(),
        }
    }
}

pub struct Http {
    inner: reqwest::Client,
    url: String,
}

impl Http {
    pub fn new(host: &str, authorization: &str) -> Self {
        let mut header_map = HeaderMap::new();
        header_map.insert("Authorization", authorization.parse().unwrap());

        let builder = ClientBuilder::new().default_headers(header_map);
        let url = format!("http://{}/v4", host);

        Http {
            inner: builder.build().unwrap(),
            url,
        }
    }

    pub async fn load_tracks(&self, identifier: &str) -> Result<LoadTracksResult, Error> {
        let url = Url::parse_with_params(
            &format!("{}/loadtracks", self.url),
            &[("identifier", identifier)],
        )
        .unwrap();

        let result = self.inner.get(url).send().await;
        if result.is_err() {
            return Err(Error::HttpRequestFailed);
        }

        result
            .unwrap()
            .json::<LoadTracksResult>()
            .await
            .map_err(|_| Error::HttpRequestFailed)
    }
}
