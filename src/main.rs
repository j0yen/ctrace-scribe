//! ctrace-scribe — single-pass ctrace NDJSON summary renderer with backfill.
//!
//! Subcommands:
//! - `scribe render <log.ndjson> [--out PATH]`  — render one session log
//! - `scribe backfill <dir> [--dry-run] [--force]` — bulk idempotent render
//! - `scribe rollup [--dir DIR] [--since WHEN] [--top N] [--format md|json]`
//!   — cross-session daily trace digest

use clap::{Parser, Subcommand};
use std::io::Write as IoWrite;

pub(crate) mod backfill;
pub(crate) mod parser;
pub(crate) mod render;
pub(crate) mod rollup;

/// Single-pass ctrace NDJSON summary renderer + backfill + cross-session rollup.
#[derive(Parser, Debug)]
#[command(
    name = "scribe",
    version,
    about = "Render ctrace session logs to Markdown summaries",
    long_about = "ctrace-scribe renders ctrace NDJSON session logs to Markdown summaries \
                  in a single streaming pass. Use `render` for one file, `backfill` to \
                  idempotently close gaps across a directory, or `rollup` to emit a \
                  cross-session daily digest across all logs in a time window."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Render one NDJSON log to its .summary.md
    Render(render::RenderArgs),
    /// Render all *.ndjson in a directory that lack a *.summary.md
    Backfill(backfill::BackfillArgs),
    /// Emit a cross-session digest across all logs in a time window
    Rollup(rollup::RollupArgs),
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Render(args) => render::run(&args),
        Command::Backfill(args) => backfill::run(&args),
        Command::Rollup(args) => rollup::run(&args),
    };
    if let Err(e) = result {
        let _ = std::io::stderr().write_all(format!("error: {e}\n").as_bytes());
        std::process::exit(1);
    }
}
