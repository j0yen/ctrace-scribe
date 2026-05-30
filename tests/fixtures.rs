//! Shared test fixture helpers.
//! NOT an acceptance test file — imported by acceptance_* modules.

use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A minimal valid ctrace NDJSON event with the given type and fields.
pub fn execve_event(ts: u64, file: &str, pid: u64) -> String {
    format!(r#"{{"ts":{ts},"type":"execve","file":"{file}","pid":{pid}}}"#)
}

pub fn openat_event(ts: u64, path: &str) -> String {
    format!(r#"{{"ts":{ts},"type":"openat","path":"{path}"}}"#)
}

pub fn unlinkat_event(ts: u64, path: &str) -> String {
    format!(r#"{{"ts":{ts},"type":"unlinkat","path":"{path}"}}"#)
}

pub fn connect_event(ts: u64, comm: &str) -> String {
    format!(r#"{{"ts":{ts},"type":"connect","comm":"{comm}"}}"#)
}

/// Write lines to a temp file and return its path and the TempDir (must keep alive).
pub fn write_ndjson(dir: &Path, name: &str, lines: &[String]) -> PathBuf {
    let path = dir.join(name);
    let content = lines.join("\n") + "\n";
    std::fs::write(&path, content).expect("write fixture");
    path
}

/// Create a temp directory.
pub fn tmpdir() -> TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// A minimal log with a variety of event types.
pub fn minimal_log_lines() -> Vec<String> {
    vec![
        execve_event(1000, "/usr/bin/bash", 100),
        execve_event(2000, "/usr/bin/bash", 101),
        execve_event(3000, "/usr/bin/git", 102),
        openat_event(4000, "/home/jsy/brain/journal/test.md"),
        openat_event(5000, "/usr/lib/libc.so"),
        unlinkat_event(6000, "/tmp/scratch.txt"),
        connect_event(7000, "curl"),
        connect_event(8000, "curl"),
    ]
}

/// A log with a flagged SSH write.
pub fn flagged_ssh_log_lines() -> Vec<String> {
    let mut lines = minimal_log_lines();
    lines.push(openat_event(9000, "/home/jsy/.ssh/known_hosts"));
    lines
}

/// A log with no flagged writes.
pub fn no_flagged_log_lines() -> Vec<String> {
    minimal_log_lines()
}
