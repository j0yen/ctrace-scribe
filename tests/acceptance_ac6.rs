//! AC6: backfill run twice renders 0 on the second run (idempotent).

mod fixtures;
use fixtures::*;

#[test]
fn acceptance_ac6_backfill_is_idempotent() {
    let dir = tmpdir();
    write_ndjson(dir.path(), "session.ndjson", &minimal_log_lines());

    // First run — renders 1.
    let first = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["backfill", dir.path().to_str().expect("dir path")])
        .output()
        .expect("first backfill");

    assert!(first.status.success(), "first backfill failed");
    let first_out = String::from_utf8_lossy(&first.stdout);
    assert!(
        first_out.contains("rendered 1"),
        "expected rendered 1 on first run: {first_out}"
    );

    // Second run — renders 0.
    let second = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["backfill", dir.path().to_str().expect("dir path")])
        .output()
        .expect("second backfill");

    assert!(second.status.success(), "second backfill failed");
    let second_out = String::from_utf8_lossy(&second.stdout);
    assert!(
        second_out.contains("rendered 0"),
        "expected rendered 0 on second run: {second_out}"
    );
}
