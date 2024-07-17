use std::env;

use lofty::file::AudioFile;
use lofty::probe::Probe;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::task;
use tokio::time::Instant;
use walkdir::WalkDir;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Track {
    id: f64,
    name: String,
    track_name: String,
    artist_name: String,
    album_name: String,
    duration: f64,
    instrumental: bool,
    plain_lyrics: Option<String>,
    synced_lyrics: Option<String>,
}

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id: {}, name: {}, track name: {}, artist name: {}, album name: {}, duration: {}, instrumental: {}, plain lyrics available: {}, synced lyrics available: {}" ,  self.id, self.name, self.track_name, self.artist_name, self.album_name, self.duration, self.instrumental, self.plain_lyrics.is_some(), self.synced_lyrics.is_some())
    }
}

#[tokio::main]
async fn main() {
    let start_time = Instant::now();
    let args: Vec<String> = env::args().collect();

    let music_dir = match args.get(1) {
        Some(dir) => Path::new(dir),
        None => {
            println!("Usage: {} <music_directory>", args[0]);
            return;
        }
    };

    if !music_dir.is_dir() {
        eprintln!("Error: '{}' is not a valid directory.", music_dir.display());
        return;
    }

    let audio_extensions = [
        ".mp3", ".flac", ".ogg", ".wav", ".aac", ".m4a", ".wma", ".opus", ".ape",
    ];

    let successful_count = Arc::new(AtomicUsize::new(0));
    let failed_count = Arc::new(AtomicUsize::new(0));

    let walker = WalkDir::new(&music_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && audio_extensions.iter().any(|&ext| {
                    e.path()
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .ends_with(ext)
                })
        })
        .map(|entry| {
            let music_dir = music_dir.to_owned();
            println!("{}", &entry.path().display());
            let successful_count = Arc::clone(&successful_count);
            let failed_count = Arc::clone(&failed_count);
            task::spawn(async move {
                parse_song_path(&entry.path(), &music_dir, successful_count, failed_count).await;
            })
        });

    // Collect all tasks into a Vec and await their completion
    let tasks: Vec<_> = walker.collect();
    for task in tasks {
        task.await.unwrap()
    }

    println!(
        "Successful tasks: {}",
        successful_count.load(Ordering::SeqCst)
    );
    println!("Failed tasks: {}", failed_count.load(Ordering::SeqCst));
    println!(
        "Total tasks: {}",
        successful_count.load(Ordering::SeqCst) + failed_count.load(Ordering::SeqCst)
    );
    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);
    println!("Time taken: {:?}", elapsed_time);
}

async fn parse_song_path(
    file_path: &Path,
    music_dir: &Path,
    successful_count: Arc<AtomicUsize>,
    failed_count: Arc<AtomicUsize>,
) -> Option<(String, String, String)> {
    if let Some(album_dir) = file_path.parent() {
        if let Some(artist_dir) = album_dir.parent() {
            if let Some(music_dirr) = artist_dir.parent() {
                if music_dirr.starts_with(music_dir) {
                    // Extract names from the directory structure
                    let artist = artist_dir.file_name()?.to_string_lossy().into_owned();
                    let album = album_dir.file_name()?.to_string_lossy().into_owned();
                    let song = file_path.file_stem()?.to_string_lossy().into_owned();
                    let clean_song = remove_numbered_prefix(&song);

                    let mut full_path = PathBuf::from("");
                    full_path.push(music_dir);
                    full_path.push(artist_dir);
                    full_path.push(album_dir);
                    full_path.push(file_path);

                    let duration = get_audio_duration(&full_path);

                    let mut url = "http://lrclib.net/api/get".to_string();
                    url.push_str("?track_name=");
                    url.push_str(&urlencoding::encode(&clean_song));
                    url.push_str("&artist_name=");
                    url.push_str(&urlencoding::encode(&artist));
                    url.push_str("&album_name=");
                    url.push_str(&urlencoding::encode(&album));
                    url.push_str("&duration=");
                    url.push_str(&duration.as_secs().to_string());
                    // url.push_str("?q=");
                    // let query = format!(
                    //     "{}+{}+{}",
                    //     encode(&album).replace("%20", "+"),
                    //     encode(&artist).replace("%20", "+"),
                    //     encode(&clean_song).replace("%20", "+")
                    // );
                    // Fetch JSON data from the API
                    // url.push_str(&query);

                    let url = url.replace("%20", "+");
                    println!("url: {}", url);
                    let response = reqwest::get(url).await;
                    match response {
                        Ok(resp) => {
                            let json_data = resp.text().await;
                            match json_data {
                                Ok(json) => {
                                    // println!("json: {}", json);
                                    // Deserialize the JSON response
                                    let mut tracks: Vec<Track> = Vec::new();

                                    let track = serde_json::from_str(&json);

                                    match track {
                                        Ok(track) => {
                                            tracks.push(track);

                                            // Find the first track with non-empty syncedLyrics
        
                                            if let Some(track) =
                                                tracks.iter().find(|&t| t.synced_lyrics.is_some())
                                            {
                                                match &track.synced_lyrics {
                                                    Some(lyrics) => {
                                                        // println!(
                                                        //     "First track with synced lyrics: {:?}",
                                                        //     lyrics
                                                        // );
                                                        let file_name = format!(
                                                            "{}/{}/{}/{}.lrc",
                                                            music_dir.display(),
                                                            artist,
                                                            album,
                                                            song
                                                        );
        
                                                        // Create a new file or overwrite existing one
                                                        let mut file = File::create(&file_name)
                                                            .await
                                                            .expect(&format!(
                                                                "Failed to create file {}",
                                                                file_name
                                                            ));
        
                                                        // Write syncedLyrics to the file
                                                        file.write_all(lyrics.as_bytes()).await.expect(
                                                            &format!(
                                                                "Failed to write to file {}",
                                                                file_name
                                                            ),
                                                        );
        
                                                        println!(
                                                            "Saved lyrics for {} to {}",
                                                            track.name, file_name
                                                        );
                                                        successful_count.fetch_add(1, Ordering::SeqCst);
                                                    }
                                                    None => {
                                                        failed_count.fetch_add(1, Ordering::SeqCst);
                                                    }
                                                }
                                            } else {
                                                println!(
                                                    "No track with synced lyrics found for {}, found at {}",
                                                    song,
                                                    file_path.display()
                                                );
                                                failed_count.fetch_add(1, Ordering::SeqCst);
                                            }
                                        },
                                        Err(_) => {
                                            println!("Failed to format data {} for song {}", json, song);
                                            failed_count.fetch_add(1, Ordering::SeqCst);
                                        } ,
                                    }

                                  
                                }
                                Err(_) => {
                                    failed_count.fetch_add(1, Ordering::SeqCst);
                                }
                            }
                        }
                        Err(_) => {
                            failed_count.fetch_add(1, Ordering::SeqCst);
                        }
                    }

                    return Some((artist, album, song));
                }
            }
        }
    }
    None
}

fn remove_numbered_prefix(s: &str) -> String {
    // Find the index of the first dot
    if let Some(index) = s.find('.') {
        // Check if characters before the dot are digits
        if s[..index].chars().all(|c| c.is_digit(10)) {
            // Return substring after the dot
            return s[index + 1..].trim().to_string();
        }
    }
    // If no valid prefix found, return the original string
    s.to_string()
}

fn get_audio_duration(file_path: &PathBuf) -> Duration {
    let tagged_file = Probe::open(file_path)
        .expect("ERROR: Bad path provided!")
        .read()
        .expect("ERROR: Failed to read file!");

    tagged_file.properties().duration()
}
