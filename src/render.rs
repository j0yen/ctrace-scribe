//! `scribe render` — single-pass NDJSON → Markdown renderer.

use crate::parser::{Accumulators, parse_log};
use clap::Args;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{self, Write as IoWrite};
use std::path::{Path, PathBuf};

/// Arguments for the `render` subcommand.
#[derive(Args, Debug)]
pub struct RenderArgs {
    /// Path to the ctrace NDJSON session log.
    pub log: PathBuf,

    /// Output path for the summary Markdown.
    /// Defaults to `<log>.summary.md` next to the input.
    /// Use `-` to write to stdout.
    #[arg(long, short)]
    pub out: Option<PathBuf>,

    /// Emit pass-count and event-count stats to stderr.
    #[arg(long, default_value_t = false)]
    pub stats: bool,
}

/// Run the render subcommand.
///
/// # Errors
/// Returns an error if the log file cannot be read or the output cannot be written.
pub fn run(args: &RenderArgs) -> Result<(), String> {
    let log_path = &args.log;
    if !log_path.exists() {
        return Err(format!(
            "log file not found: {}",
            log_path.display()
        ));
    }
    if !log_path.is_file() {
        return Err(format!(
            "not a file: {}",
            log_path.display()
        ));
    }

    let accs = parse_log(log_path)?;

    if args.stats {
        io::stderr()
            .write_all(
                format!(
                    "stats: file_opens=1 total_events={} malformed={}\n",
                    accs.total_events, accs.malformed_count
                )
                .as_bytes(),
            )
            .map_err(|e| format!("stderr write: {e}"))?;
    }

    let md = render_markdown(&accs, log_path);

    match args.out.as_deref() {
        Some(p) if p == Path::new("-") => {
            io::stdout()
                .write_all(md.as_bytes())
                .map_err(|e| format!("stdout write: {e}"))?;
        }
        Some(out_path) => {
            fs::write(out_path, &md).map_err(|e| format!("write {}: {e}", out_path.display()))?;
        }
        None => {
            let out_path = summary_path(log_path);
            fs::write(&out_path, &md)
                .map_err(|e| format!("write {}: {e}", out_path.display()))?;
            io::stdout()
                .write_all(format!("{}\n", out_path.display()).as_bytes())
                .map_err(|e| format!("stdout write: {e}"))?;
        }
    }

    Ok(())
}

/// Derive `<log>.summary.md` from `<log>.ndjson`.
#[must_use]
pub fn summary_path(log: &Path) -> PathBuf {
    let stem = log
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("session");
    let parent = log.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!("{stem}.summary.md"))
}

/// Format duration from milliseconds into a human-readable string.
fn format_duration(duration_ms: u64) -> String {
    // Convert ms to whole seconds (integer arithmetic, no floats).
    let total_secs = duration_ms / 1000;
    if total_secs == 0 {
        let ms = duration_ms;
        if ms == 0 {
            "0s".to_owned()
        } else {
            format!("{ms}ms")
        }
    } else if total_secs < 60 {
        let frac = duration_ms % 1000;
        let tenths = frac / 100;
        format!("{total_secs}.{tenths}s")
    } else {
        let m = total_secs / 60;
        let s = total_secs % 60;
        format!("{m}m {s}s")
    }
}

/// Render accumulators to a Markdown string.
#[must_use]
pub fn render_markdown(accs: &Accumulators, log_path: &Path) -> String {
    let mut out = String::with_capacity(2048);

    // Title
    writeln!(out, "# Claude session trace summary").ok();
    writeln!(out).ok();

    // Log line
    writeln!(out, "- Log: `{}`", log_path.display()).ok();

    // Duration / event / PID / write count line
    let dur_str = format_duration(accs.duration_ms);
    writeln!(
        out,
        "- Duration {dur_str} · {} events · {} PIDs · {} writes",
        accs.total_events, accs.unique_pid_count, accs.total_writes
    )
    .ok();

    if accs.malformed_count > 0 {
        writeln!(out, "- ({} malformed lines skipped)", accs.malformed_count).ok();
    }

    // Top binaries executed
    writeln!(out).ok();
    writeln!(out, "## Top binaries executed").ok();
    writeln!(out).ok();
    if accs.exec_histogram.is_empty() {
        writeln!(out, "(none)").ok();
    } else {
        writeln!(out, "```").ok();
        let mut execs: Vec<(&String, &u64)> = accs.exec_histogram.iter().collect();
        execs.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        for (file, count) in execs.iter().take(10) {
            writeln!(out, "{count:>7} {file}").ok();
        }
        writeln!(out, "```").ok();
    }

    // Writes outside expected scope
    writeln!(out).ok();
    writeln!(out, "## Writes outside expected scope").ok();
    writeln!(out).ok();
    if accs.out_of_scope_paths.is_empty() {
        writeln!(out, "(none)").ok();
    } else {
        writeln!(out, "```").ok();
        let mut paths: Vec<&String> = accs.out_of_scope_paths.iter().collect();
        paths.sort();
        for p in paths.iter().take(30) {
            writeln!(out, "{p}").ok();
        }
        writeln!(out, "```").ok();
    }

    // Flagged sensitive-path writes (conditional)
    if !accs.flagged_paths.is_empty() {
        writeln!(out).ok();
        writeln!(out, "## ⚠ Flagged sensitive-path writes").ok();
        writeln!(out).ok();
        writeln!(out, "```").ok();
        let mut flagged: Vec<&String> = accs.flagged_paths.iter().collect();
        flagged.sort();
        for p in flagged.iter().take(20) {
            writeln!(out, "{p}").ok();
        }
        writeln!(out, "```").ok();
    }

    // Deletions
    writeln!(out).ok();
    writeln!(out, "## Deletions").ok();
    writeln!(out).ok();
    if accs.deleted_paths.is_empty() {
        writeln!(out, "(none)").ok();
    } else {
        writeln!(out, "```").ok();
        let mut deletes: Vec<&String> = accs.deleted_paths.iter().collect();
        deletes.sort();
        for p in deletes.iter().take(20) {
            writeln!(out, "{p}").ok();
        }
        writeln!(out, "```").ok();
    }

    // Outbound connect() by process
    writeln!(out).ok();
    writeln!(out, "## Outbound connect() by process").ok();
    writeln!(out).ok();
    if accs.connect_histogram.is_empty() {
        writeln!(out, "(none)").ok();
    } else {
        writeln!(out, "```").ok();
        let mut connects: Vec<(&String, &u64)> = accs.connect_histogram.iter().collect();
        connects.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        for (comm, count) in connects.iter().take(10) {
            writeln!(out, "{count:>7} {comm}").ok();
        }
        writeln!(out, "```").ok();
    }

    out
}
