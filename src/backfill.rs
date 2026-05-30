//! `scribe backfill` — idempotent bulk render for a directory of NDJSON logs.

use crate::render::{RenderArgs, run as render_run, summary_path};
use clap::Args;
use std::io::{self, Write as IoWrite};
use std::path::PathBuf;
use std::time::SystemTime;

/// Arguments for the `backfill` subcommand.
#[derive(Args, Debug)]
pub struct BackfillArgs {
    /// Directory to scan for `*.ndjson` files.
    pub dir: PathBuf,

    /// List what would be rendered without writing any files.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Re-render even if a summary already exists (when ndjson is newer than summary).
    #[arg(long, default_value_t = false)]
    pub force: bool,
}

/// Run the backfill subcommand.
///
/// # Errors
/// Returns an error if the directory cannot be read.
pub fn run(args: &BackfillArgs) -> Result<(), String> {
    let dir = &args.dir;
    if !dir.exists() {
        return Err(format!("directory not found: {}", dir.display()));
    }
    if !dir.is_dir() {
        return Err(format!("not a directory: {}", dir.display()));
    }

    // Collect all *.ndjson files in the directory (non-recursive).
    let mut ndjson_files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("read_dir {}: {e}", dir.display()))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("ndjson") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    ndjson_files.sort();

    let mut rendered: u64 = 0;
    let mut skipped: u64 = 0;
    let mut stdout = io::stdout();

    for log_path in &ndjson_files {
        let summary = summary_path(log_path);

        let needs_render = if summary.exists() {
            if args.force {
                // Force: re-render if ndjson is newer than summary.
                let ndjson_mtime = mtime(log_path);
                let summary_mtime = mtime(&summary);
                match (ndjson_mtime, summary_mtime) {
                    (Some(n), Some(s)) => n > s,
                    _ => true, // If we can't get mtimes, render to be safe.
                }
            } else {
                false
            }
        } else {
            true
        };

        if needs_render {
            if args.dry_run {
                stdout
                    .write_all(format!("would-render: {}\n", log_path.display()).as_bytes())
                    .map_err(|e| format!("stdout write: {e}"))?;
            } else {
                let render_args = RenderArgs {
                    log: log_path.clone(),
                    out: None,
                    stats: false,
                };
                render_run(&render_args)?;
                stdout
                    .write_all(format!("rendered: {}\n", log_path.display()).as_bytes())
                    .map_err(|e| format!("stdout write: {e}"))?;
            }
            rendered += 1;
        } else {
            skipped += 1;
        }
    }

    if args.dry_run {
        stdout
            .write_all(format!("dry-run: would-render {rendered}, skipped {skipped}\n").as_bytes())
            .map_err(|e| format!("stdout write: {e}"))?;
    } else {
        stdout
            .write_all(format!("rendered {rendered}, skipped {skipped}\n").as_bytes())
            .map_err(|e| format!("stdout write: {e}"))?;
    }

    Ok(())
}

fn mtime(path: &std::path::Path) -> Option<SystemTime> {
    path.metadata().ok()?.modified().ok()
}
