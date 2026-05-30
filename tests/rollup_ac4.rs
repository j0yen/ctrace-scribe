//! Rollup AC4: `--top N` caps each histogram section to N entries; default is documented in --help.

mod fixtures;
use fixtures::*;

#[test]
fn rollup_ac4_top_n_caps_histogram_entries() {
    let dir = tmpdir();

    // Build a log with 30 distinct binaries so we can verify --top 5 cuts at 5.
    let mut lines = Vec::new();
    for i in 0..30u64 {
        lines.push(format!(
            r#"{{"ts":{i},"type":"execve","file":"/usr/bin/tool{i}","pid":{i}}}"#
        ));
    }
    write_ndjson(dir.path(), "session.ndjson", &lines);

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args([
            "rollup",
            "--dir",
            dir.path().to_str().expect("dir path"),
            "--top",
            "5",
            "--format",
            "json",
        ])
        .output()
        .expect("run scribe rollup --top 5 --format json");

    assert!(
        output.status.success(),
        "rollup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("json");
    let bins = parsed["top_binaries_executed"]
        .as_array()
        .expect("array");
    assert!(
        bins.len() <= 5,
        "expected ≤5 entries with --top 5, got {}",
        bins.len()
    );
}

#[test]
fn rollup_ac4_help_mentions_rollup_flags() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["rollup", "--help"])
        .output()
        .expect("run scribe rollup --help");

    assert!(output.status.success(), "rollup --help must exit 0");
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(
        help.contains("--top"),
        "help must mention --top, got:\n{help}"
    );
    assert!(
        help.contains("--since"),
        "help must mention --since, got:\n{help}"
    );
    assert!(
        help.contains("--format"),
        "help must mention --format, got:\n{help}"
    );
}
