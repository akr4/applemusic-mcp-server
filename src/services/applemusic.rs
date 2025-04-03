use log::{debug, error};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use rmcp::{model::*, tool, Error as McpError, ServerHandler};
use serde::Deserialize;

use crate::models::*;

#[derive(Debug, Deserialize)]
struct AppleMusicSearchResponse {
    results: AppleMusicSearchResults,
}

#[derive(Debug, Deserialize)]
struct AppleMusicSearchResults {
    #[serde(default)]
    songs: Option<AppleMusicSongData>,
}

#[derive(Debug, Deserialize)]
struct AppleMusicSongData {
    data: Vec<AppleMusicSong>,
}

#[derive(Debug, Deserialize)]
struct AppleMusicSong {
    id: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    type_name: String,
    attributes: AppleMusicSongAttributes,
}

#[derive(Debug, Deserialize)]
struct AppleMusicSongAttributes {
    name: String,
    #[serde(rename = "artistName")]
    artist_name: String,
    #[serde(rename = "albumName")]
    album_name: String,
    #[serde(rename = "durationInMillis")]
    duration_in_millis: u64,
    artwork: AppleMusicArtwork,
    url: String,
}

#[derive(Debug, Deserialize)]
struct AppleMusicArtwork {
    url: String,
    width: u32,
    height: u32,
}

#[derive(Clone)]
pub struct AppleMusicServer {
    client: reqwest::Client,
    developer_token: String,
    storefront: String,
}

#[tool(tool_box)]
impl AppleMusicServer {
    pub fn new(
        team_id: Option<String>,
        key_id: Option<String>,
        private_key_path: Option<String>,
        storefront: String,
    ) -> Self {
        // Get authentication information from command line arguments
        let developer_token = match (team_id, key_id, private_key_path) {
            (Some(team_id), Some(key_id), Some(private_key_path)) => {
                debug!("Generating token from command line arguments");
                let generator =
                    crate::auth::DeveloperTokenGenerator::new(team_id, key_id, private_key_path);
                match generator.generate_token(12) {
                    Ok(token) => {
                        debug!("Generated developer token from command line arguments");
                        token
                    }
                    Err(e) => {
                        error!("Developer token generation error: {}", e);
                        String::new()
                    }
                }
            }
            _ => {
                error!("Missing required command line arguments: --team-id, --key-id, --private-key-path");
                std::process::exit(1)
            }
        };

        // Create HTTP client
        let client = reqwest::Client::new();

        Self {
            client,
            developer_token,
            storefront,
        }
    }

    /// Build headers for request
    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        if !self.developer_token.is_empty() {
            let auth_value = format!("Bearer {}", self.developer_token);
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value).unwrap());
        }

        headers
    }

    #[tool(description = "Search for songs from Apple Music by title, album name, or artist name")]
    async fn search_songs(
        &self,
        #[tool(aggr)] request: SearchSongsRequest,
    ) -> Result<CallToolResult, McpError> {
        debug!("Searching for songs: {}", request.query);

        if self.developer_token.is_empty() {
            return Err(McpError::internal_error(
                "Apple Music developer token is not set",
                None,
            ));
        }

        let url = format!(
            "https://api.music.apple.com/v1/catalog/{}/search",
            self.storefront
        );

        let headers = self.build_headers();

        let params = [
            ("term", request.query.as_str()),
            ("types", "songs"),
            ("limit", "25"),
        ];

        let response = match self
            .client
            .get(&url)
            .headers(headers)
            .query(&params)
            .send()
            .await
        {
            Ok(res) => res,
            Err(e) => {
                error!("Apple Music API call error: {}", e);
                return Err(McpError::internal_error(
                    format!("API call failed: {}", e),
                    None,
                ));
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!(
                "Apple Music API error: Status: {}, Response: {}",
                status, error_text
            );
            return Err(McpError::internal_error(
                format!("API error: Status: {}", status),
                Some(serde_json::Value::String(error_text)),
            ));
        }

        let apple_response: AppleMusicSearchResponse = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                error!("JSON parse error: {}", e);
                return Err(McpError::internal_error(
                    format!("JSON parse error: {}", e),
                    None,
                ));
            }
        };

        let songs = if let Some(songs_data) = apple_response.results.songs {
            songs_data
                .data
                .into_iter()
                .map(|song| {
                    // Replace {w}x{h} in artwork URL with actual size
                    let artwork_url = song
                        .attributes
                        .artwork
                        .url
                        .replace("{w}", &song.attributes.artwork.width.to_string())
                        .replace("{h}", &song.attributes.artwork.height.to_string());

                    Song {
                        id: song.id,
                        title: song.attributes.name,
                        artist: song.attributes.artist_name,
                        album: song.attributes.album_name,
                        duration_ms: song.attributes.duration_in_millis,
                        artwork_url,
                        apple_music_url: song.attributes.url,
                    }
                })
                .collect()
        } else {
            // Empty array if no search results
            vec![]
        };

        let content = match Content::json(songs) {
            Ok(content) => content,
            Err(e) => return Err(e),
        };

        Ok(CallToolResult::success(vec![content]))
    }

    #[tool(description = "Generate Deep Link for playback")]
    async fn generate_playback_link(
        &self,
        #[tool(aggr)] request: GeneratePlaybackLinkRequest,
    ) -> Result<CallToolResult, McpError> {
        debug!("Generating playback link");

        if request.song_id.is_none() && request.playlist_id.is_none() {
            return Err(McpError::invalid_params(
                "Either song_id or playlist_id is required",
                None,
            ));
        }

        let url = if let Some(song_id) = request.song_id {
            format!("https://music.apple.com/song/{}", song_id)
        } else if let Some(playlist_id) = request.playlist_id {
            format!("https://music.apple.com/playlist/{}", playlist_id)
        } else {
            unreachable!()
        };

        let response = PlaybackLinkResponse {
            apple_music_url: url,
        };

        let json = match serde_json::to_value(response) {
            Ok(val) => val,
            Err(e) => {
                return Err(McpError::invalid_params(
                    format!("Serialization error: {}", e),
                    None,
                ))
            }
        };

        let content = match Content::json(json) {
            Ok(content) => content,
            Err(e) => return Err(e),
        };

        Ok(CallToolResult::success(vec![content]))
    }
}

#[tool(tool_box)]
impl ServerHandler for AppleMusicServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Apple Music MCP Server: Provides features like song search, playlist creation, etc.".to_string()),
        }
    }
}
