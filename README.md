# Lyrics getter

Huge thanks to <https://github.com/tranxuanthang/lrclib> ofc.

Uses lrclib.net to get lyrics for my Jellyfin library. Does /get, if unavailable tried to do /search

Is very much dependent on having the Jellyfin suggested music library structure. (Artist/Album/Song).

To run go `lyricsrs <music_directory>` or clone the repo and `cargo run <music_directory>`.

Will not overwrite any .lrc files you already have with the existing name by default.

Only does synced lyrics by default because they are cool.

## Flags

`lyricsrs` accepts command-line flags to change its behaviour:

- `--overwrite`: Overwrite lyrics files, if present, with lyrics from lrclib
- `--allow-plain`: Allow writing plain lyrics if no synced lyrics are available
