//! Rollup AC1: `scribe rollup --dir <fixture-dir>` emits a Markdown digest with all
//! required sections: session count, Top write-path prefixes, Top binaries executed,
//! Outbound connect by process, Deletions, and a Flagged section (lines or "none").

mod fixtures;
use fixtures::*;

#[test]
fn rollup_ac1_md_digest_contains_all_sections() {
    let dir = tmpdir();
    // Write a fixture log with a variety of event types.
    write_ndjson(dir.path(), "session1.ndjson", &minimal_log_lines());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["rollup", "--dir", dir.path().to_str().expect("dir path")])
        .output()
        .expect("run scribe rollup");

    assert!(
        output.status.success(),
        "rollup exited non-zero: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Session count must be present
    assert!(
        stdout.contains("Sessions in window"),
        "missing 'Sessions in window' in:\n{stdout}"
    );
    // Top write-path prefixes
    assert!(
        stdout.contains("## Top write-path prefixes"),
        "missing 'Top write-path prefixes' section in:\n{stdout}"
    );
    // Top binaries executed
    assert!(
        stdout.contains("## Top binaries executed"),
        "missing 'Top binaries executed' section in:\n{stdout}"
    );
    // Outbound connect by process
    assert!(
        stdout.contains("## Outbound connect() by process"),
        "missing 'Outbound connect() by process' section in:\n{stdout}"
    );
    // Deletions
    assert!(
        stdout.contains("## Deletions"),
        "missing 'Deletions' section in:\n{stdout}"
    );
    // Flagged section — present with "none" when no flagged writes
    assert!(
        stdout.contains("## Flagged sensitive-path writes"),
        "missing 'Flagged sensitive-path writes' section in:\n{stdout}"
    );
    assert!(
        stdout.contains("none"),
        "no-flagged digest should say 'none' in:\n{stdout}"
    );
}
