use std::path::PathBuf;
use temp_dir::TempDir;

use tokio::fs;

pub const SONGS: [&str; 11] = [
    "Taylor Swift/THE TORTURED POETS DEPARTMENT: THE ANTHOLOGY/1. Fortnight (Ft. Post Malone).flac",
    "Taylor Swift/THE TORTURED POETS DEPARTMENT: THE ANTHOLOGY/2. The Tortured Poets Department.mp3",
    "Taylor Swift/THE TORTURED POETS DEPARTMENT: THE ANTHOLOGY/3. My Boy Only Breaks His Favorite Toys.m4a",
    "Taylor Swift/reputation/2. End Game.flac",
    "Lou Reed/The Best of Lou Reed/8. Perfect Day.m4a",
    "Heilung/Drif/01 Asja.flac",
    "Heilung/Drif/02 - Anoana.flac",
    "LINKIN PARK/Hybrid Theory/09-A Place for my Head.mp3",
    "LINKIN PARK/LIVING THINGS/6.CASTLE OF GLASS.flac",
    "Our Lady Peace/Clumsy/5_4AM.mp3",
    "Our Lady Peace/Spiritual Machines/04 _ In Repair.mp3",
];

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

pub async fn setup(basedir: &TempDir) {
    let mut tasks = vec![];

    for song_path in SONGS.iter() {
        let path = basedir.child(song_path);

        tasks.push(tokio::spawn(async move {
            create_files_with_names(&path).await;
        }));
    }

    // Await all tasks to complete
    for task in tasks {
        task.await.expect("Failed to execute task");
    }
    println!(
        "Files created successfully in folder: {}",
        basedir.path().display()
    );
}

pub async fn check_lrcs(basedir: &TempDir) -> bool {
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
        let song_path = basedir.child(file);
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
