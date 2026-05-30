//! ctrace-scribe — single-pass ctrace NDJSON summary renderer with backfill.
//!
//! Subcommands:
//! - `scribe render <log.ndjson> [--out PATH]`  — render one session log
//! - `scribe backfill <dir> [--dry-run] [--force]` — bulk idempotent render

use clap::{Parser, Subcommand};
use std::io::Write as IoWrite;

pub(crate) mod backfill;
pub(crate) mod parser;
pub(crate) mod render;

/// Single-pass ctrace NDJSON summary renderer + backfill.
#[derive(Parser, Debug)]
#[command(
    name = "scribe",
    version,
    about = "Render ctrace session logs to Markdown summaries",
    long_about = "ctrace-scribe renders ctrace NDJSON session logs to Markdown summaries \
                  in a single streaming pass. Use `render` for one file, `backfill` to \
                  idempotently close gaps across a directory."
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
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Render(args) => render::run(&args),
        Command::Backfill(args) => backfill::run(&args),
    };
    if let Err(e) = result {
        let _ = std::io::stderr().write_all(format!("error: {e}\n").as_bytes());
        std::process::exit(1);
    }
}
