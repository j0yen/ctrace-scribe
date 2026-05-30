//! Rollup AC6: `scribe rollup` over a directory of ≥300 fixture logs completes
//! without error and in bounded memory (no per-file subprocess, no ARG_MAX).

mod fixtures;
use fixtures::*;

#[test]
fn rollup_ac6_handles_300_logs_without_error() {
    let dir = tmpdir();

    // Generate 300 small NDJSON logs.
    for i in 0..300usize {
        let lines = vec![
            format!(r#"{{"ts":{i},"type":"execve","file":"/usr/bin/tool{i}","pid":{i}}}"#),
            format!(r#"{{"ts":{i},"type":"openat","path":"/home/jsy/brain/note{i}.md"}}"#),
            format!(r#"{{"ts":{i},"type":"connect","comm":"curl"}}"#),
        ];
        write_ndjson(dir.path(), &format!("session_{i:04}.ndjson"), &lines);
    }

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["rollup", "--dir", dir.path().to_str().expect("dir path")])
        .output()
        .expect("run scribe rollup");

    assert!(
        output.status.success(),
        "rollup failed on 300 logs: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Must report 300 sessions in window.
    assert!(
        stdout.contains("Sessions in window: **300**"),
        "expected 300 sessions, got:\n{stdout}"
    );
}
