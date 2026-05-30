//! AC7: backfill --dry-run writes no files and exits 0, listing would-render set.

mod fixtures;
use fixtures::*;

#[test]
fn acceptance_ac7_dry_run_writes_no_files() {
    let dir = tmpdir();
    let log = write_ndjson(dir.path(), "session.ndjson", &minimal_log_lines());
    let summary = dir.path().join("session.summary.md");
    let _ = &log; // suppress unused warning

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["backfill", "--dry-run", dir.path().to_str().expect("dir path")])
        .output()
        .expect("run scribe backfill --dry-run");

    assert!(
        output.status.success(),
        "dry-run should exit 0 but got: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // No summary file should have been written.
    assert!(
        !summary.exists(),
        "dry-run must not write summary file, but {} exists",
        summary.display()
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should list the would-render file.
    assert!(
        stdout.contains("would-render"),
        "expected 'would-render' in dry-run output: {stdout}"
    );
    assert!(
        stdout.contains("session.ndjson"),
        "expected log name in dry-run output: {stdout}"
    );
}
