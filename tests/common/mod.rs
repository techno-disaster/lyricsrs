use std::{io::Error, path::PathBuf, str::FromStr};

use tokio::fs;

pub const SONGS: [&str; 5] = [
    "Taylor Swift/THE TORTURED POETS DEPARTMENT: THE ANTHOLOGY/1. Fortnight (Ft. Post Malone).flac",
    "Taylor Swift/THE TORTURED POETS DEPARTMENT: THE ANTHOLOGY/2. The Tortured Poets Department.mp3",
    "Taylor Swift/THE TORTURED POETS DEPARTMENT: THE ANTHOLOGY/3. My Boy Only Breaks His Favorite Toys.m4a",
    "Taylor Swift/reputation/2. End Game.flac",
    "Lou Reed/The Best of Lou Reed/8. Perfect Day.m4a",
];

pub const BASE_DIR: &str = "Music";

async fn create_files_with_names(output_file: &PathBuf) {
    let dirs = output_file.parent().expect("could not parse dirs");
    let file_name = output_file.file_name().expect("could not parse name");

    let _dirs = fs::create_dir_all(dirs).await;

    let format = file_name
        .to_str()
        .expect("hm wtf")
        .split(".")
        .last()
        .expect("could not get extension");

    let source_file = match format {
        "flac" => "tests/data/template.flac",
        "mp3" => "tests/data/template.mp3",
        "m4a" => "tests/data/template.m4a",
        _ => todo!(),
    };

    let _copy = fs::copy(PathBuf::from(source_file), output_file).await;
}

pub async fn setup() {
    let mut tasks = vec![];

    for song_path in SONGS.iter() {
        let mut path = PathBuf::from(BASE_DIR);
        path.push(song_path);

        tasks.push(tokio::spawn(async move {
            create_files_with_names(&path).await;
        }));
    }

    // Await all tasks to complete
    for task in tasks {
        task.await.expect("Failed to execute task");
    }
    println!("Files created successfully in folder: {}", BASE_DIR);
}

pub async fn cleanup() {
    let _remove = fs::remove_dir_all(BASE_DIR).await.expect("cleanup failed");
}

pub async fn check_lrcs() -> bool {
    let mut song_names: Vec<String> = Vec::new();

    SONGS.iter().for_each(|song_path| {
        let song_name = song_path
            .split_at(song_path.rfind(".").expect("invalid song path"))
            .0
            .to_owned()
            + ".lrc";

        song_names.push(song_name);
    });
    let mut all_exist = true;
    for file in song_names.iter() {
        let mut song_path = std::env::current_dir().expect("could not get curr dir");
        song_path.push(BASE_DIR);
        song_path.push(file);
        if let Err(_) = fs::metadata(song_path.clone()).await {
            println!("file {} does not exist", song_path.to_string_lossy());
            all_exist = false;
            break;
        } else {
            println!("Found lrc file {}", song_path.to_string_lossy());
        }
    }

    all_exist
}
