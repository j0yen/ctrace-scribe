//! AC9: --help documents render and backfill with their flags; exits 0.

#[test]
fn acceptance_ac9_help_documents_render_and_backfill() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .arg("--help")
        .output()
        .expect("run scribe --help");

    assert!(
        output.status.success(),
        "--help should exit 0 but got status: {}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("render") || stdout.contains("Render"),
        "--help should document 'render' subcommand in:\n{stdout}"
    );
    assert!(
        stdout.contains("backfill") || stdout.contains("Backfill"),
        "--help should document 'backfill' subcommand in:\n{stdout}"
    );
}

#[test]
fn acceptance_ac9_render_help_documents_flags() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["render", "--help"])
        .output()
        .expect("run scribe render --help");

    assert!(
        output.status.success(),
        "render --help should exit 0 but got status: {}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--out") || stdout.contains("-o"),
        "render --help should document --out flag in:\n{stdout}"
    );
}

#[test]
fn acceptance_ac9_backfill_help_documents_flags() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["backfill", "--help"])
        .output()
        .expect("run scribe backfill --help");

    assert!(
        output.status.success(),
        "backfill --help should exit 0 but got status: {}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--dry-run"),
        "backfill --help should document --dry-run flag in:\n{stdout}"
    );
    assert!(
        stdout.contains("--force"),
        "backfill --help should document --force flag in:\n{stdout}"
    );
}
