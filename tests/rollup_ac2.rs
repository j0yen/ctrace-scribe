//! Rollup AC2: `--since today` includes only logs with mtime on the current local day;
//! a log dated yesterday is excluded and a verifying test proves it.

mod fixtures;
use fixtures::*;

use filetime::FileTime;
use std::time::{Duration, SystemTime};

#[test]
fn rollup_ac2_since_today_excludes_yesterday() {
    let dir = tmpdir();

    // Write today's log (mtime = now, default).
    write_ndjson(dir.path(), "today.ndjson", &minimal_log_lines());

    // Write a log that we will back-date to yesterday.
    let yesterday_path =
        write_ndjson(dir.path(), "yesterday.ndjson", &minimal_log_lines());

    // Set mtime to 36 hours ago — safely in yesterday.
    let yesterday_mtime = SystemTime::now() - Duration::from_secs(36 * 3_600);
    let ft = FileTime::from_system_time(yesterday_mtime);
    filetime::set_file_mtime(&yesterday_path, ft).expect("set mtime");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args([
            "rollup",
            "--dir",
            dir.path().to_str().expect("dir path"),
            "--since",
            "today",
        ])
        .output()
        .expect("run scribe rollup");

    assert!(
        output.status.success(),
        "rollup exited non-zero: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Only 1 session should be included (today's log).
    assert!(
        stdout.contains("Sessions in window: **1**"),
        "expected exactly 1 session (yesterday excluded), got:\n{stdout}"
    );
}
