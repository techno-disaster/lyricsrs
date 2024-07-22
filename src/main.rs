use std::env;

use lofty::file::AudioFile;
use lofty::probe::Probe;
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
use urlencoding::encode;
use walkdir::WalkDir;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
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

    let mut total_count = 0;
    let successful_count = Arc::new(AtomicUsize::new(0));

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
            total_count += 1;
            task::spawn(async move {
                parse_song_path(&entry.path(), &music_dir, successful_count).await;
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
    println!(
        "Failed tasks: {}",
        total_count - successful_count.load(Ordering::SeqCst)
    );
    println!("Total tasks: {}", total_count,);
    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);
    println!("Time taken: {:?}", elapsed_time);
}

async fn parse_song_path(file_path: &Path, music_dir: &Path, successful_count: Arc<AtomicUsize>) {
    if let Some(album_dir) = file_path.parent() {
        if let Some(artist_dir) = album_dir.parent() {
            if let Some(music_dirr) = artist_dir.parent() {
                if music_dirr.starts_with(music_dir) {
                    exact_search(
                        music_dir,
                        artist_dir,
                        album_dir,
                        file_path,
                        successful_count,
                    )
                    .await;
                }
            }
        }
    }
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

async fn save_synced_lyrics(
    music_dir: &Path,
    artist_dir: &Path,
    album_dir: &Path,
    song_name: &String,
    synced_lyrics: String,
    successful_count: Arc<AtomicUsize>,
) {
    let mut parent_dir = PathBuf::new();
    parent_dir.push(music_dir);
    parent_dir.push(artist_dir);
    parent_dir.push(album_dir);

    let file_path = format!("{}/{}.lrc", parent_dir.to_string_lossy(), song_name);

    // Create a new file or overwrite existing one
    let mut file = File::create(&file_path)
        .await
        .expect(&format!("Failed to create file {}", file_path));

    // Write syncedLyrics to the file
    file.write_all(synced_lyrics.as_bytes())
        .await
        .expect(&format!("Failed to write to file {}", file_path));

    println!("Saved lyrics for {} to {}", song_name, file_path);
    successful_count.fetch_add(1, Ordering::SeqCst);
}

async fn exact_search(
    music_dir: &Path,
    artist_dir: &Path,
    album_dir: &Path,
    file_path: &Path,
    successful_count: Arc<AtomicUsize>,
) {
    let artist_name = artist_dir
        .file_name()
        .expect("invalid artist_dir")
        .to_string_lossy()
        .into_owned();
    let album_name = album_dir
        .file_name()
        .expect("invalid album_dir")
        .to_string_lossy()
        .into_owned();
    // includes extenstion
    // let song_full_name = file_path
    //     .file_name()
    //     .expect("invalid file_path")
    //     .to_string_lossy()
    //     .into_owned();
    let song_name = file_path
        .file_stem()
        .expect("invalid file_path")
        .to_string_lossy()
        .into_owned();
    let clean_song = remove_numbered_prefix(&song_name);

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
    url.push_str(&urlencoding::encode(&artist_name));
    url.push_str("&album_name=");
    url.push_str(&urlencoding::encode(&album_name));
    url.push_str("&duration=");
    url.push_str(&duration.as_secs().to_string());

    let url = url.replace("%20", "+");
    println!("[exact_search] requesting: {}", url);
    let response = reqwest::get(url)
        .await
        .expect("[exact_search] request failed");
    let json_data = response
        .text()
        .await
        .expect("[exact_search] parsing body failed");

    let track: Result<Track, serde_json::Error> = serde_json::from_str(&json_data);
    match track {
        Ok(track) => {
            // Find the first track with non-empty syncedLyrics
            match &track.synced_lyrics {
                Some(lyrics) => {
                    save_synced_lyrics(
                        &music_dir,
                        artist_dir,
                        album_dir,
                        &song_name,
                        lyrics.clone(),
                        successful_count,
                    )
                    .await;
                }
                None => {
                    println!(
                        "[exact_search] synced lyrics unavilable {} for song {} falling back to fuzzy_search",
                        json_data, clean_song
                    );
                    fuzzy_search(
                        music_dir,
                        artist_dir,
                        album_dir,
                        file_path,
                        successful_count,
                    )
                    .await;
                }
            }
        }
        Err(_) => {
            println!(
                "[exact_search] could not parse track response {} for song {} falling back to fuzzy_search",
                json_data, clean_song
            );
            fuzzy_search(
                music_dir,
                artist_dir,
                album_dir,
                file_path,
                successful_count,
            )
            .await;
        }
    }
}

async fn fuzzy_search(
    music_dir: &Path,
    artist_dir: &Path,
    album_dir: &Path,
    file_path: &Path,
    successful_count: Arc<AtomicUsize>,
) {
    let artist_name = artist_dir
        .file_name()
        .expect("invalid artist_dir")
        .to_string_lossy()
        .into_owned();
    let album_name = album_dir
        .file_name()
        .expect("invalid album_dir")
        .to_string_lossy()
        .into_owned();
    // includes extenstion
    // let song_full_name = file_path
    //     .file_name()
    //     .expect("invalid file_path")
    //     .to_string_lossy()
    //     .into_owned();
    let song_name = file_path
        .file_stem()
        .expect("invalid file_path")
        .to_string_lossy()
        .into_owned();
    let clean_song = remove_numbered_prefix(&song_name);
    let mut url = "http://lrclib.net/api/search?q=".to_string();

    // normal search and pick first one with synced lyrics
    let query = format!(
        "{}+{}+{}",
        encode(&album_name),
        encode(&artist_name),
        encode(&clean_song)
    );
    url.push_str(&query);

    let url = url.replace("%20", "+");
    println!("[fuzzy_search] requesting: {}", url);
    let response = reqwest::get(url)
        .await
        .expect("[fuzzy_search] request failed");
    let json_data = response
        .text()
        .await
        .expect("[fuzzy_search] parsing body failed");

    let tracks: Result<Vec<Track>, serde_json::Error> = serde_json::from_str(&json_data);
    match tracks {
        Ok(tracks) => {
            if let Some(track) = tracks.iter().find(|&t| t.synced_lyrics.is_some()) {
                match &track.synced_lyrics {
                    Some(lyrics) => {
                        save_synced_lyrics(
                            &music_dir,
                            artist_dir,
                            album_dir,
                            &song_name,
                            lyrics.clone(),
                            successful_count,
                        )
                        .await;
                    }
                    None => {
                        println!(
                            "[fuzzy_search] could not parse synced lyrics for song {}",
                            clean_song
                        );
                    }
                }
            } else {
                println!(
                    "[fuzzy_search] could not find synced lyrics for song {}",
                    clean_song
                );
            }
        }
        Err(_) => {
            println!(
                "[fuzzy_search] failed to parse json {} for {}",
                json_data, clean_song
            );
        }
    }
}
