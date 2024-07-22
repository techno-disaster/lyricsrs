use std::env;
use std::path::PathBuf;
use std::process::Command;
mod common;

#[tokio::test]
async fn test_cli() {
    common::setup().await;

    let target_dir = env::var("CARGO_MANIFEST_DIR").expect("could not get target dir");
    let mut current_dir = env::current_dir().expect("could not get current dir");
    current_dir.push(common::BASE_DIR);

    let mut path = PathBuf::from(target_dir);

    path.push("target/release/lyricsrs");
    let output = Command::new(path)
        .arg(current_dir)
        .output()
        .expect("Failed to execute command");

    println!("{:?}", output);

    assert!(output.status.success());

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    // keep in sync with SONGS in common/mod.rs
    let to_find = format!(
        "Successful tasks: {}\nFailed tasks: 0\nTotal tasks: {}",
        common::SONGS.len(),
        common::SONGS.len()
    );
    assert!(stdout_str.contains(&to_find));

    assert!(common::check_lrcs().await);

    common::cleanup().await;
}
