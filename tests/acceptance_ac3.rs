//! AC3: render on a large fixture completes in <= 2s and uses a single pass.
//!
//! We generate a 50k-event fixture (synthetic but large enough to validate
//! performance). The PRD's 124k-event file isn't in the repo, so we generate
//! a proportionally large fixture. The pass-count assertion is structural:
//! --stats reports file_opens=1.

mod fixtures;
use fixtures::*;
use std::time::Instant;

fn generate_large_log(event_count: usize) -> Vec<String> {
    let mut lines = Vec::with_capacity(event_count);
    for i in 0..event_count {
        let ts = (i as u64) * 10;
        let binary = match i % 5 {
            0 => "/usr/bin/bash",
            1 => "/usr/bin/python3",
            2 => "/usr/bin/cargo",
            3 => "/usr/bin/rustc",
            _ => "/usr/bin/git",
        };
        lines.push(execve_event(ts, binary, (i as u64) % 1000 + 1));
        if i % 3 == 0 {
            lines.push(openat_event(ts + 1, "/home/jsy/brain/notes.md"));
        }
    }
    lines
}

#[test]
fn acceptance_ac3_large_fixture_completes_in_two_seconds() {
    let dir = tmpdir();
    // 50k exec events + ~16k openat events = ~66k total lines
    let lines = generate_large_log(50_000);
    let log = write_ndjson(dir.path(), "large_session.ndjson", &lines);

    let start = Instant::now();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args([
            "render",
            "--stats",
            log.to_str().expect("log path"),
        ])
        .output()
        .expect("run scribe render");
    let elapsed = start.elapsed();

    assert!(
        output.status.success(),
        "render failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        elapsed.as_secs_f64() <= 2.0,
        "render took {:.2}s (> 2s limit)",
        elapsed.as_secs_f64()
    );

    // Verify single pass via --stats output
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("file_opens=1"),
        "expected file_opens=1 in stats but got: {stderr}"
    );
}
