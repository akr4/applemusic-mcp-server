use rmcp::schemars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Song {
    #[schemars(description = "Song ID")]
    pub id: String,

    #[schemars(description = "Song title")]
    pub title: String,

    #[schemars(description = "Artist name")]
    pub artist: String,

    #[schemars(description = "Album name")]
    pub album: String,

    #[schemars(description = "Song duration (milliseconds)")]
    pub duration_ms: u64,

    #[schemars(description = "Artwork URL")]
    pub artwork_url: String,

    #[schemars(description = "Apple Music URL")]
    pub apple_music_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Playlist {
    #[schemars(description = "Playlist ID")]
    pub id: String,

    #[schemars(description = "Playlist name")]
    pub name: String,

    #[schemars(description = "Songs included in the playlist")]
    pub songs: Vec<Song>,

    #[schemars(description = "Apple Music URL")]
    pub apple_music_url: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchSongsRequest {
    #[schemars(description = "Search query (song title, album name, artist name)")]
    pub query: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GeneratePlaybackLinkRequest {
    #[schemars(description = "Song ID string (mutually exclusive with playlist_id)")]
    pub song_id: Option<String>,

    #[schemars(description = "Playlist ID string (mutually exclusive with song_id)")]
    pub playlist_id: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct PlaybackLinkResponse {
    #[schemars(description = "Apple Music URL")]
    pub apple_music_url: String,
}
