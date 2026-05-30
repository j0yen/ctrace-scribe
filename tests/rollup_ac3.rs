//! Rollup AC3: `--format json` emits a single valid JSON object parseable with known keys;
//! `--format md` is the default.

mod fixtures;
use fixtures::*;

#[test]
fn rollup_ac3_format_json_is_valid_json_with_required_keys() {
    let dir = tmpdir();
    write_ndjson(dir.path(), "session.ndjson", &minimal_log_lines());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args([
            "rollup",
            "--dir",
            dir.path().to_str().expect("dir path"),
            "--format",
            "json",
        ])
        .output()
        .expect("run scribe rollup --format json");

    assert!(
        output.status.success(),
        "rollup --format json exited non-zero: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("JSON output must be parseable");

    let obj = parsed.as_object().expect("top-level must be a JSON object");

    assert!(
        obj.contains_key("session_count"),
        "missing 'session_count' key"
    );
    assert!(
        obj.contains_key("total_events"),
        "missing 'total_events' key"
    );
    assert!(
        obj.contains_key("top_write_path_prefixes"),
        "missing 'top_write_path_prefixes' key"
    );
    assert!(
        obj.contains_key("top_binaries_executed"),
        "missing 'top_binaries_executed' key"
    );
    assert!(
        obj.contains_key("outbound_connect_by_process"),
        "missing 'outbound_connect_by_process' key"
    );
    assert!(
        obj.contains_key("deletions"),
        "missing 'deletions' key"
    );
    assert!(
        obj.contains_key("flagged_sensitive_writes"),
        "missing 'flagged_sensitive_writes' key"
    );
}

#[test]
fn rollup_ac3_default_format_is_md() {
    let dir = tmpdir();
    write_ndjson(dir.path(), "session.ndjson", &minimal_log_lines());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["rollup", "--dir", dir.path().to_str().expect("dir path")])
        .output()
        .expect("run scribe rollup (default format)");

    assert!(output.status.success(), "rollup failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Markdown output starts with a # header
    assert!(
        stdout.starts_with('#'),
        "default format should be Markdown (starts with #), got:\n{stdout}"
    );
}
