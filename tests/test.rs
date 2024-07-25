use std::env;
use std::path::PathBuf;
use std::process::Command;
use temp_dir::TempDir;
mod common;

#[tokio::test]
async fn test_cli() {
    // TempDir deletes the created directory when the struct is dropped. Call TempDir::leak() to
    // keep it around for debugging purposes.
    let tmpdir = &TempDir::new().unwrap();
    common::setup(tmpdir).await;

    let target_dir = env::var("CARGO_MANIFEST_DIR").expect("could not get target dir");

    let mut path = PathBuf::from(target_dir);
    path.push("target/release/lyricsrs");

    let output = Command::new(path)
        .arg(tmpdir.path())
        .output()
        .expect("Failed to execute command");

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    println!("Exit code: {}", output.status.code().unwrap());
    println!("STDOUT: {}", stdout_str);
    println!("STDERR: {}", stderr_str);

    assert!(output.status.success());

    // keep in sync with SONGS in common/mod.rs
    let to_find = format!(
        "Successful tasks: {}\nFailed tasks: 0\nTotal tasks: {}",
        common::SONGS.len(),
        common::SONGS.len()
    );
    assert!(stdout_str.contains(&to_find));

    assert!(common::check_lrcs(tmpdir).await);
}

#[tokio::test]
async fn test_cli_plain_lyrics_allowed() {
    // TempDir deletes the created directory when the struct is dropped. Call TempDir::leak() to
    // keep it around for debugging purposes.
    let tmpdir = &TempDir::new().unwrap();
    common::setup(tmpdir).await;

    let target_dir = env::var("CARGO_MANIFEST_DIR").expect("could not get target dir");

    let mut path = PathBuf::from(target_dir);
    path.push("target/release/lyricsrs");

    let output = Command::new(path)
        .arg("--allow-plain")
        .arg(tmpdir.path())
        .output()
        .expect("Failed to execute command");

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    println!("Exit code: {}", output.status.code().unwrap());
    println!("STDOUT: {:?}", stdout_str);
    println!("STDERR: {:?}", stderr_str);

    assert!(output.status.success());

    // keep in sync with SONGS in common/mod.rs
    let to_find = format!(
        "Successful tasks: {}\nFailed tasks: 0\nTotal tasks: {}",
        common::SONGS.len(),
        common::SONGS.len()
    );
    assert!(stdout_str.contains(&to_find));

    assert!(common::check_lrcs(tmpdir).await);
}
