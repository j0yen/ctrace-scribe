//! AC4: A truncated final line does not fail the render; summary is produced and
//! reports "1 malformed line skipped".

mod fixtures;
use fixtures::*;

#[test]
fn acceptance_ac4_truncated_final_line_is_handled_gracefully() {
    let dir = tmpdir();
    // Build a log with a truncated last line (no closing brace).
    let mut lines = minimal_log_lines();
    lines.push(r#"{"ts":9999,"type":"execve","file":"/usr/bin/truncated"#.to_owned());

    let log = write_ndjson(dir.path(), "truncated.ndjson", &lines);
    let summary = dir.path().join("truncated.summary.md");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["render", log.to_str().expect("log path")])
        .output()
        .expect("run scribe render");

    assert!(
        output.status.success(),
        "render should succeed on truncated log but got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(summary.exists(), "summary not created");

    let content = std::fs::read_to_string(&summary).expect("read summary");
    assert!(
        content.contains("1 malformed line"),
        "expected '1 malformed line skipped' note in:\n{content}"
    );
}
