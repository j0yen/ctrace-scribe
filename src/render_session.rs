//! `scribe render-session` — idempotent single-session render safe for hook use.
//!
//! Renders a session log to its `.summary.md` only if the summary is absent or
//! differs from what would be produced. Safe to call from the SessionEnd hook or
//! during backfill — the same render engine is used in both paths.

use crate::parser::parse_log;
use crate::render::{render_markdown, summary_path};
use clap::Args;
use std::fs;
use std::io::{self, Write as IoWrite};
use std::path::{Path, PathBuf};

/// Arguments for the `render-session` subcommand.
#[derive(Args, Debug)]
pub struct RenderSessionArgs {
    /// Path to the ctrace NDJSON session log (must end in `.ndjson`).
    pub log: PathBuf,

    /// Maximum seconds to spend rendering before giving up (default: 30).
    /// NOTE: the timeout is advisory — it is enforced by the calling hook via
    /// `timeout(1)`.  This flag is accepted for forward-compatibility but is not
    /// wired to an internal timer in this version.
    #[arg(long, default_value_t = 30)]
    pub timeout: u64,
}

/// Render `log` to `<log>.summary.md` idempotently.
///
/// Returns `true` if the summary was written (new or updated), `false` if the
/// existing summary was already byte-identical (no-op).
///
/// # Errors
///
/// Returns a `String` error if:
/// - `log` does not exist or is not a file.
/// - The log cannot be parsed.
/// - The summary file cannot be written.
pub fn render_session(log: &Path) -> Result<bool, String> {
    if !log.exists() {
        return Err(format!("log file not found: {}", log.display()));
    }
    if !log.is_file() {
        return Err(format!("not a file: {}", log.display()));
    }

    let accs = parse_log(log)?;
    let md = render_markdown(&accs, log);
    let out_path = summary_path(log);

    // Idempotency check: skip write if file already contains the exact bytes.
    if out_path.exists() {
        match fs::read(&out_path) {
            Ok(existing) if existing == md.as_bytes() => return Ok(false),
            _ => {}
        }
    }

    fs::write(&out_path, md.as_bytes())
        .map_err(|e| format!("write {}: {e}", out_path.display()))?;

    Ok(true)
}

/// Run the `render-session` subcommand.
///
/// Exits 0 whether the summary was written or was already up to date.
///
/// # Errors
///
/// Returns an error string if the render fails (log missing, parse error, I/O
/// error writing the summary).
pub fn run(args: &RenderSessionArgs) -> Result<(), String> {
    match render_session(&args.log)? {
        true => {
            let out_path = summary_path(&args.log);
            io::stdout()
                .write_all(format!("{}\n", out_path.display()).as_bytes())
                .map_err(|e| format!("stdout write: {e}"))?;
        }
        false => {
            // Already up to date — silent success (no stdout output).
        }
    }
    Ok(())
}
