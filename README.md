# Apple Music MCP Server

An MCP server for interacting with the Apple Music API. This server provides tools for searching songs, generating playback links.

## Features

- Search for songs by title, artist, or album
- Generate deep links for songs and playlists
- Simple command-line interface
- Uses the Model Context Protocol (MCP) to interact with clients

## Requirements

- Rust (nightly, because of rmcp)
- Apple Developer account with Apple Music API access
- Apple Music API Key (.p8 file)

## Installation

```bash
# Build the project
cargo install --path .
```

## Usage

The Apple Music MCP server requires authentication credentials to access the Apple Music API. These are provided via command-line arguments.

```bash
cargo run -- --team-id YOUR_TEAM_ID --key-id YOUR_KEY_ID --private-key-path /path/to/AuthKey.p8
```

### Command-line Arguments

```
Usage: applemusic-mcp-server [OPTIONS]

Options:
--team-id <TEAM_ID>                    Apple Developer Team ID
--key-id <KEY_ID>                      Apple Music Key ID
--private-key-path <PRIVATE_KEY_PATH>  Path to the Apple Music private key file (.p8)
--storefront <STOREFRONT>              Storefront for Apple Music (e.g. us, jp) [default: jp]
-h, --help                                 Print help
-V, --version                              Print version
```

## API Tools

### searchSongs

Search for songs from Apple Music by title, album name, or artist name.

Parameters:
- `query`: (string) - The search query text

Returns:
- Array of Song objects with details including title, artist, album, duration, and URLs

### generatePlaybackLink

Generate a deep link for playback of a song or playlist.

Parameters:
- Either `song_id` or `playlist_id` must be provided

Returns:
- Object containing the Apple Music URL
