//! AC10: Rendering a non-existent log exits non-zero with a usage error;
//! an empty dir backfills 0 and exits 0.

#[test]
fn acceptance_ac10_render_nonexistent_log_exits_nonzero() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["render", "/tmp/does-not-exist-ctrace-scribe-ac10.ndjson"])
        .output()
        .expect("run scribe render on nonexistent log");

    assert!(
        !output.status.success(),
        "render on nonexistent log should exit non-zero but succeeded"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("not found"),
        "expected error message in stderr but got:\n{stderr}"
    );
}

#[test]
fn acceptance_ac10_backfill_empty_dir_renders_zero_exits_zero() {
    let dir = tempfile::tempdir().expect("tempdir");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["backfill", dir.path().to_str().expect("dir path")])
        .output()
        .expect("run scribe backfill on empty dir");

    assert!(
        output.status.success(),
        "backfill of empty dir should exit 0 but got: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("rendered 0"),
        "expected 'rendered 0' for empty dir backfill in:\n{stdout}"
    );
}
