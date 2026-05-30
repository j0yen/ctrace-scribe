//! Single-pass NDJSON parser for ctrace session logs.
//!
//! Reads the file line by line (one pass), accumulating statistics into
//! [`Accumulators`]. Malformed/truncated lines are counted and skipped —
//! never fatal. This is the property the shell-script version lacks.

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Paths under these prefixes are considered "in scope" for Claude writes.
/// Everything else surfaces in the "Writes outside expected scope" section.
/// Matches `summarize-ctrace-session.sh`'s `in_scope` regex.
const IN_SCOPE_PREFIXES: &[&str] = &[
    "/home/jsy/projects/",
    "/home/jsy/Notes/",
    "/home/jsy/brain/",
    "/tmp/",
    "/home/jsy/.cache/",
    "/home/jsy/.claude/",
    "/home/jsy/.local/",
    "/dev/",
    "/proc/",
    "/sys/",
    "/run/user/",
    "/var/tmp/",
    "/home/jsy/wintermute/",
];

/// Paths matching these prefixes are always flagged as sensitive.
/// Matches `summarize-ctrace-session.sh`'s `flag_paths` regex.
const FLAG_PREFIXES: &[&str] = &[
    "/etc/",
    "/home/jsy/.ssh/",
    "/home/jsy/.aws/",
    "/home/jsy/.gnupg/",
    "/root/",
    "/home/jsy/Notes/Journals/Accounts.md",
];

/// All statistics accumulated during a single-pass parse.
#[derive(Debug, Default)]
pub struct Accumulators {
    /// Total non-empty lines in the log.
    pub total_events: u64,
    /// Count of lines that failed JSON parsing or lacked expected fields.
    pub malformed_count: u64,
    /// Min/max timestamp (boot-relative ms). Used for duration.
    pub ts_min: Option<u64>,
    /// Max timestamp (`ts_max` - `ts_min` = duration).
    pub ts_max: Option<u64>,
    /// Duration in milliseconds (`ts_max` - `ts_min`).
    pub duration_ms: u64,
    /// Histogram: execve'd file → count.
    pub exec_histogram: HashMap<String, u64>,
    /// Set of openat paths outside in-scope prefixes (not flagged).
    pub out_of_scope_paths: HashSet<String>,
    /// Set of flagged openat paths.
    pub flagged_paths: HashSet<String>,
    /// Set of unlinkat paths.
    pub deleted_paths: HashSet<String>,
    /// Histogram: connect comm → count.
    pub connect_histogram: HashMap<String, u64>,
    /// Unique PIDs seen in execve events.
    exec_pids: HashSet<u64>,
    /// Unique PID count.
    pub unique_pid_count: u64,
    /// Total openat events (all paths).
    pub total_writes: u64,
}

/// Parse a ctrace NDJSON log file in a single streaming pass.
///
/// # Errors
/// Returns an error if the file cannot be opened for reading.
pub fn parse_log(path: &Path) -> Result<Accumulators, String> {
    let file =
        std::fs::File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let reader = BufReader::new(file);
    let mut accs = Accumulators::default();

    for line_result in reader.lines() {
        let line = line_result.map_err(|e| format!("read {}: {e}", path.display()))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        accs.total_events += 1;

        // Parse JSON; skip malformed lines gracefully.
        let v: serde_json::Value = if let Ok(v) = serde_json::from_str(line) {
            v
        } else {
            accs.malformed_count += 1;
            continue;
        };

        // Update timestamp range.
        if let Some(ts) = v.get("ts").and_then(serde_json::Value::as_u64) {
            match (accs.ts_min, accs.ts_max) {
                (None, _) => {
                    accs.ts_min = Some(ts);
                    accs.ts_max = Some(ts);
                }
                (Some(mn), Some(mx)) => {
                    if ts < mn {
                        accs.ts_min = Some(ts);
                    }
                    if ts > mx {
                        accs.ts_max = Some(ts);
                    }
                }
                (Some(_), None) => {
                    accs.ts_max = Some(ts);
                }
            }
        }

        let event_type = v
            .get("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");

        match event_type {
            "execve" => {
                if let Some(file) = v.get("file").and_then(serde_json::Value::as_str) {
                    *accs.exec_histogram.entry(file.to_owned()).or_insert(0) += 1;
                }
                if let Some(pid) = v.get("pid").and_then(serde_json::Value::as_u64) {
                    accs.exec_pids.insert(pid);
                }
            }
            "openat" => {
                if let Some(path_str) = v.get("path").and_then(serde_json::Value::as_str) {
                    accs.total_writes += 1;
                    if is_flagged(path_str) {
                        accs.flagged_paths.insert(path_str.to_owned());
                    } else if !is_in_scope(path_str) {
                        accs.out_of_scope_paths.insert(path_str.to_owned());
                    }
                }
            }
            "unlinkat" => {
                if let Some(path_str) = v.get("path").and_then(serde_json::Value::as_str) {
                    accs.deleted_paths.insert(path_str.to_owned());
                }
            }
            "connect" => {
                if let Some(comm) = v.get("comm").and_then(serde_json::Value::as_str) {
                    *accs.connect_histogram.entry(comm.to_owned()).or_insert(0) += 1;
                }
            }
            _ => {
                // Unknown event types are silently counted in total_events
                // but not accumulated into any histogram.
            }
        }
    }

    // Compute derived fields.
    accs.duration_ms = match (accs.ts_min, accs.ts_max) {
        (Some(mn), Some(mx)) => mx.saturating_sub(mn),
        _ => 0,
    };
    accs.unique_pid_count = u64::try_from(accs.exec_pids.len()).unwrap_or(u64::MAX);

    Ok(accs)
}

fn is_flagged(path: &str) -> bool {
    FLAG_PREFIXES.iter().any(|prefix| path.starts_with(prefix))
}

fn is_in_scope(path: &str) -> bool {
    IN_SCOPE_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_flagged_ssh() {
        assert!(is_flagged("/home/jsy/.ssh/id_rsa"));
    }

    #[test]
    fn test_is_flagged_etc() {
        assert!(is_flagged("/etc/passwd"));
    }

    #[test]
    fn test_not_flagged_home() {
        assert!(!is_flagged("/home/jsy/projects/foo"));
    }

    #[test]
    fn test_in_scope_cache() {
        assert!(is_in_scope("/home/jsy/.cache/something"));
    }

    #[test]
    fn test_in_scope_wintermute() {
        assert!(is_in_scope("/home/jsy/wintermute/ctrace-scribe/src/main.rs"));
    }

    #[test]
    fn test_not_in_scope_usr() {
        assert!(!is_in_scope("/usr/bin/ls"));
    }
}
