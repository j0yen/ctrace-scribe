//! Rollup AC5: a flagged write (e.g. /home/jsy/.ssh/known_hosts) in any session log
//! appears under the Flagged section naming the source log; with no flagged writes the
//! section reads "none".

mod fixtures;
use fixtures::*;

#[test]
fn rollup_ac5_flagged_write_appears_with_session_name() {
    let dir = tmpdir();
    // One session log with an SSH flagged write.
    write_ndjson(dir.path(), "session_ssh.ndjson", &flagged_ssh_log_lines());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["rollup", "--dir", dir.path().to_str().expect("dir path")])
        .output()
        .expect("run scribe rollup");

    assert!(
        output.status.success(),
        "rollup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("## Flagged sensitive-path writes"),
        "missing Flagged section in:\n{stdout}"
    );
    // The source log filename must appear in the flagged section.
    assert!(
        stdout.contains("session_ssh.ndjson"),
        "source log name not listed in Flagged section:\n{stdout}"
    );
    // The path itself must appear.
    assert!(
        stdout.contains(".ssh/known_hosts"),
        "flagged path not listed in Flagged section:\n{stdout}"
    );
}

#[test]
fn rollup_ac5_no_flagged_writes_says_none() {
    let dir = tmpdir();
    write_ndjson(dir.path(), "session_clean.ndjson", &no_flagged_log_lines());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["rollup", "--dir", dir.path().to_str().expect("dir path")])
        .output()
        .expect("run scribe rollup");

    assert!(output.status.success(), "rollup failed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("## Flagged sensitive-path writes"),
        "Flagged section must always be present:\n{stdout}"
    );
    assert!(
        stdout.contains("none"),
        "Flagged section should say 'none' when no flagged writes:\n{stdout}"
    );
}
