//! Rollup AC8: `--help` documents `rollup` and its flags; exit 0.

#[test]
fn rollup_ac8_help_exits_zero() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["rollup", "--help"])
        .output()
        .expect("run scribe rollup --help");

    assert!(
        output.status.success(),
        "rollup --help must exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Must mention all flags
    assert!(stdout.contains("--dir"), "help missing --dir");
    assert!(stdout.contains("--since"), "help missing --since");
    assert!(stdout.contains("--top"), "help missing --top");
    assert!(stdout.contains("--format"), "help missing --format");
}

#[test]
fn rollup_ac8_top_level_help_mentions_rollup() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["--help"])
        .output()
        .expect("run scribe --help");

    assert!(output.status.success(), "scribe --help must exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("rollup"),
        "top-level help must mention rollup subcommand:\n{stdout}"
    );
}
