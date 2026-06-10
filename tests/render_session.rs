//! Tests for `scribe render-session` — idempotency, fallback, and error handling.

mod fixtures;
use fixtures::*;
use std::fs;

// ── helpers ──────────────────────────────────────────────────────────────────

fn scribe() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
}

fn summary_path_for(log: &std::path::Path) -> std::path::PathBuf {
    // mirrors render::summary_path: strip .ndjson, append .summary.md
    let stem = log
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("session");
    let parent = log.parent().unwrap_or_else(|| std::path::Path::new("."));
    parent.join(format!("{stem}.summary.md"))
}

// ── AC1: idempotency ─────────────────────────────────────────────────────────

/// Rendering the same log twice must:
///  - succeed both times (exit 0)
///  - produce the same bytes both times
///  - NOT update the mtime on the second call (file is already byte-identical)
#[test]
fn render_session_idempotent() {
    let dir = tmpdir();
    let log = write_ndjson(dir.path(), "session.ndjson", &minimal_log_lines());
    let summary = summary_path_for(&log);

    // First call — should write the summary.
    let out1 = scribe()
        .args(["render-session", log.to_str().expect("log path")])
        .output()
        .expect("run scribe render-session (1)");
    assert!(
        out1.status.success(),
        "first render-session failed: {}",
        String::from_utf8_lossy(&out1.stderr)
    );
    assert!(summary.exists(), "summary not created after first call");

    let bytes1 = fs::read(&summary).expect("read summary after first call");
    let mtime1 = summary
        .metadata()
        .expect("stat summary")
        .modified()
        .expect("mtime");

    // Second call — should be a no-op (exit 0, same bytes).
    let out2 = scribe()
        .args(["render-session", log.to_str().expect("log path")])
        .output()
        .expect("run scribe render-session (2)");
    assert!(
        out2.status.success(),
        "second render-session failed: {}",
        String::from_utf8_lossy(&out2.stderr)
    );

    let bytes2 = fs::read(&summary).expect("read summary after second call");
    assert_eq!(bytes1, bytes2, "summary bytes changed on idempotent call");

    let mtime2 = summary
        .metadata()
        .expect("stat summary")
        .modified()
        .expect("mtime");
    assert_eq!(mtime1, mtime2, "mtime changed on idempotent render (file re-written)");
}

// ── AC2: missing-marker fallback ─────────────────────────────────────────────

/// When no `claude-owns.json` marker is present the subcommand still renders
/// correctly when given the log path directly. This test validates the "caller
/// passes the path, not the marker" contract — the subcommand itself never reads
/// the marker; the hook shell script is responsible for the fallback lookup.
#[test]
fn render_session_no_marker_still_renders() {
    let dir = tmpdir();
    let log = write_ndjson(dir.path(), "session.ndjson", &minimal_log_lines());
    let summary = summary_path_for(&log);

    // Confirm no marker file exists.
    assert!(!dir.path().join("claude-owns.json").exists());

    let out = scribe()
        .args(["render-session", log.to_str().expect("log path")])
        .output()
        .expect("run scribe render-session");
    assert!(
        out.status.success(),
        "render-session failed without marker: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(summary.exists(), "summary not created when marker absent");

    let content = fs::read_to_string(&summary).expect("read summary");
    assert!(content.contains("## Top binaries executed"), "summary malformed");
}

// ── AC3: missing log — exit 1, no panic ──────────────────────────────────────

/// Passing a non-existent path must produce a non-zero exit and an error message
/// on stderr, but must NOT panic (no SIGABRT, no `thread 'main' panicked`).
#[test]
fn render_session_missing_log_exits_nonzero_no_panic() {
    let dir = tmpdir();
    let nonexistent = dir.path().join("does_not_exist.ndjson");

    let out = scribe()
        .args(["render-session", nonexistent.to_str().expect("path")])
        .output()
        .expect("run scribe render-session");

    assert!(
        !out.status.success(),
        "expected non-zero exit for missing log, got success"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("thread 'main' panicked"),
        "panic detected in stderr: {stderr}"
    );
    assert!(
        stderr.contains("error:"),
        "expected 'error:' in stderr, got: {stderr}"
    );
}

// ── AC4: hook-branch simulation ───────────────────────────────────────────────

/// Simulate each of the four hook branches by constructing the equivalent
/// `scribe render-session` invocations and verifying all exit 0 (or 1 for
/// genuine errors — the *hook* wrapper always exits 0, not the subcommand itself
/// when the log is truly missing).
///
/// Branch A — marker present, log present   → exit 0, summary written.
/// Branch B — marker absent, log present    → exit 0, summary written.
/// Branch C — log missing                   → exit 1 (hook wraps with `|| true`).
/// Branch D — log is not a file (dir)       → exit 1 (hook wraps with `|| true`).
#[test]
fn hook_branches_render_session_contract() {
    let dir = tmpdir();

    // Branch A: normal path — log exists.
    {
        let log = write_ndjson(dir.path(), "branch_a.ndjson", &minimal_log_lines());
        let out = scribe()
            .args(["render-session", log.to_str().expect("path")])
            .output()
            .expect("branch A");
        assert!(out.status.success(), "branch A failed");
    }

    // Branch B: no marker file — same as branch A from scribe's perspective.
    {
        let log = write_ndjson(dir.path(), "branch_b.ndjson", &minimal_log_lines());
        // Ensure no marker alongside the log.
        assert!(!log.parent().unwrap().join("claude-owns.json").exists());
        let out = scribe()
            .args(["render-session", log.to_str().expect("path")])
            .output()
            .expect("branch B");
        assert!(out.status.success(), "branch B failed");
    }

    // Branch C: log missing — scribe exits non-zero; hook must `|| true`.
    {
        let missing = dir.path().join("missing.ndjson");
        let out = scribe()
            .args(["render-session", missing.to_str().expect("path")])
            .output()
            .expect("branch C");
        assert!(!out.status.success(), "branch C should exit non-zero");
        // No panic.
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(!stderr.contains("thread 'main' panicked"), "panic in branch C");
    }

    // Branch D: path exists but is a directory — scribe exits non-zero.
    {
        let dir_path = dir.path().join("a_directory");
        fs::create_dir(&dir_path).expect("create dir");
        let out = scribe()
            .args(["render-session", dir_path.to_str().expect("path")])
            .output()
            .expect("branch D");
        assert!(!out.status.success(), "branch D should exit non-zero");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(!stderr.contains("thread 'main' panicked"), "panic in branch D");
    }
}
