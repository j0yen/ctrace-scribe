//! AC5: backfill renders exactly the *.ndjson files missing a *.summary.md;
//! leaves existing summaries untouched; prints "rendered N, skipped M".

mod fixtures;
use fixtures::*;

#[test]
fn acceptance_ac5_backfill_renders_missing_summaries_only() {
    let dir = tmpdir();

    // Create 3 logs: two without summaries, one with an existing summary.
    let log_a = write_ndjson(dir.path(), "a.ndjson", &minimal_log_lines());
    let log_b = write_ndjson(dir.path(), "b.ndjson", &minimal_log_lines());
    let log_c = write_ndjson(dir.path(), "c.ndjson", &minimal_log_lines());

    // Pre-create a summary for c with known content.
    let summary_c = dir.path().join("c.summary.md");
    std::fs::write(&summary_c, "# pre-existing summary for c\n").expect("write pre-existing");
    let pre_existing_content =
        std::fs::read_to_string(&summary_c).expect("read pre-existing");

    let _ = (&log_a, &log_b, &log_c); // suppress unused warnings

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["backfill", dir.path().to_str().expect("dir path")])
        .output()
        .expect("run scribe backfill");

    assert!(
        output.status.success(),
        "backfill failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have rendered 2 (a and b), skipped 1 (c already has summary).
    assert!(
        stdout.contains("rendered 2") && stdout.contains("skipped 1"),
        "expected 'rendered 2, skipped 1' in: {stdout}"
    );

    // Summaries for a and b should now exist.
    assert!(
        dir.path().join("a.summary.md").exists(),
        "a.summary.md not created"
    );
    assert!(
        dir.path().join("b.summary.md").exists(),
        "b.summary.md not created"
    );

    // c's summary should be UNCHANGED (still the pre-existing content).
    let after_content = std::fs::read_to_string(&summary_c).expect("read c summary after");
    assert_eq!(
        after_content, pre_existing_content,
        "c.summary.md was modified but should be untouched"
    );
}
