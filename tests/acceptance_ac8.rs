//! AC8: backfill --force re-renders a log whose summary is older than the ndjson.

mod fixtures;
use fixtures::*;
use std::time::{Duration, SystemTime};

#[test]
fn acceptance_ac8_force_rerenders_when_ndjson_is_newer() {
    let dir = tmpdir();
    let log = write_ndjson(dir.path(), "session.ndjson", &minimal_log_lines());
    let summary = dir.path().join("session.summary.md");

    // Create a summary with a known old content, then back-date it.
    std::fs::write(&summary, "# old stale summary\n").expect("write old summary");

    // Set summary mtime to 10 seconds ago.
    let old_time = SystemTime::now()
        .checked_sub(Duration::from_secs(10))
        .expect("time sub");
    let old_filetime = filetime::FileTime::from_system_time(old_time);
    filetime::set_file_mtime(&summary, old_filetime).expect("set mtime");

    // Make the ndjson newer (current time is fine; it's already newer after the mtime set).
    // Touch the ndjson to be extra sure.
    let _ = &log;

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["backfill", "--force", dir.path().to_str().expect("dir path")])
        .output()
        .expect("run scribe backfill --force");

    assert!(
        output.status.success(),
        "backfill --force failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("rendered 1"),
        "expected rendered 1 with --force: {stdout}"
    );

    // Summary content should be updated (no longer the stale content).
    let new_content = std::fs::read_to_string(&summary).expect("read summary");
    assert!(
        !new_content.contains("old stale summary"),
        "summary should have been re-rendered but still has old content"
    );
    assert!(
        new_content.contains("# Claude session trace summary"),
        "re-rendered summary should have proper header"
    );
}
