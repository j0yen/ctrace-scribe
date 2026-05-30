//! Rollup AC7: an empty or `--since`-excludes-everything window emits a well-formed
//! "0 sessions" digest and exits 0.

mod fixtures;
use fixtures::*;

use filetime::FileTime;
use std::time::{Duration, SystemTime};

#[test]
fn rollup_ac7_empty_dir_emits_zero_session_digest() {
    let dir = tmpdir();
    // No files in the directory.

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["rollup", "--dir", dir.path().to_str().expect("dir path")])
        .output()
        .expect("run scribe rollup on empty dir");

    assert!(
        output.status.success(),
        "rollup on empty dir should exit 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Sessions in window: **0**"),
        "expected 0 sessions digest, got:\n{stdout}"
    );
}

#[test]
fn rollup_ac7_since_excludes_all_logs_gives_zero_sessions() {
    let dir = tmpdir();

    // Write a log and back-date it to 8 days ago, then query --since 1d.
    let log_path = write_ndjson(dir.path(), "old.ndjson", &minimal_log_lines());
    let old_mtime = SystemTime::now() - Duration::from_secs(8 * 86_400);
    let ft = FileTime::from_system_time(old_mtime);
    filetime::set_file_mtime(&log_path, ft).expect("set mtime");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args([
            "rollup",
            "--dir",
            dir.path().to_str().expect("dir path"),
            "--since",
            "1d",
        ])
        .output()
        .expect("run scribe rollup --since 1d");

    assert!(
        output.status.success(),
        "rollup with all-excluded window should exit 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Sessions in window: **0**"),
        "expected 0 sessions (all excluded), got:\n{stdout}"
    );
}
