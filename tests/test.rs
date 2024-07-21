use std::env;
use std::path::PathBuf;
use std::process::Command;
use temp_dir::TempDir;
mod common;

#[tokio::test]
async fn test_cli() {
    let args: Vec<&str> = Vec::new();
    run_test_command(&args, false).await;
}

#[tokio::test]
async fn test_cli_plain_lyrics_allowed() {
    let mut args = Vec::new();
    args.push("--allow-plain");
    run_test_command(&args, false).await;
}

#[tokio::test]
async fn test_cli_existing_lyrics() {
    let args: Vec<&str> = Vec::new();
    run_test_command(&args, true).await;
}

#[tokio::test]
async fn test_cli_no_existing_lyrics_with_flag() {
    let mut args = Vec::new();
    args.push("--overwrite");
    run_test_command(&args, false).await;
}

#[tokio::test]
async fn test_cli_existing_lyrics_with_flag() {
    let mut args = Vec::new();
    args.push("--overwrite");
    run_test_command(&args, true).await;
}

// Generic runner for tests that only need CLI flags changed. Optionally will create a LRC file for
// validating behaviour around lyrics file replacement.
async fn run_test_command(args: &Vec<&str>, add_lrc: bool) {
    // TempDir deletes the created directory when the struct is dropped. Call TempDir::leak() to
    // keep it around for debugging purposes.
    let tmpdir = &TempDir::new().unwrap();
    common::setup(tmpdir).await;

    if add_lrc {
        let mut file_name = tmpdir.child(common::SONGS[3]);
        file_name.set_extension("lrc");
        common::create_files_with_names(&file_name).await;
    }

    let target_dir = env::var("CARGO_MANIFEST_DIR").expect("could not get target dir");

    let mut path = PathBuf::from(target_dir);
    path.push("target/release/lyricsrs");

    let mut cmd_args = Vec::new();
    cmd_args.clone_from(args);
    cmd_args.push(tmpdir.path().to_str().unwrap());
    let output = Command::new(path)
        .args(cmd_args)
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
