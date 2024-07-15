### Lyrics getter

Uses lrclib.net to get lyrics for my Jellyfin library.

Is very much dependant on having the Jellyfin suggested music library structure. (Artist/Album/Song).

To run go `lyricsrs <music_directory>` or clone the repo and `cargo run <music_directory>`.

Will overwrite any .lrc files you already have with the existing name.

Only does synced lyrics because they are cool.