# ctrace-scribe

Turns ctrace NDJSON session logs into readable Markdown summaries — one log at a time, a whole directory at once, or a cross-session daily digest — in a single streaming pass.

## Why it exists

`ctrace` records every Claude session to an NDJSON log, and a SessionEnd hook is supposed to render a one-page summary when the session ends. It does, when the session exits gracefully. Heavy headless sessions don't: cgroup teardown SIGKILLs them before the hook runs, and the log is left permanently un-summarized. `ctrace-scribe` is the rendering engine that closes that gap. The same single-pass parser drives every path — render one log, render the hook's session, backfill the ones that were missed, or roll a day's worth of sessions into one digest — so the summaries stay consistent no matter how the session ended.

The binary installs as `scribe`.

## Install

```sh
cargo install --path .
```

Or via the wintermute bootstrap installer:

```sh
curl -sSf https://raw.githubusercontent.com/j0yen/wintermute/main/bootstrap/install.sh | bash
```

Requires Rust 1.85+.

## Subcommands

```
scribe render <log.ndjson> [--out PATH]
scribe backfill <dir> [--dry-run] [--force]
scribe rollup [--dir DIR] [--since WHEN] [--top N] [--format md|json]
scribe render-session <log.ndjson> [--timeout SECS]
```

### `render` — one log to one summary

Renders a single session log to `<log>.summary.md`. The summary carries a `Log:` line, duration / event / PID / write counts, and sections for top binaries, writes outside the expected scope, deletions, outbound connections, and — only when present — flagged writes to sensitive paths. Use `--out -` for stdout.

```sh
scribe render ~/.cache/ctrace/sessions/claude-20260528T162617.ndjson
```

A truncated final line from an ungraceful exit doesn't fail the render; the summary reports `N malformed lines skipped`.

### `backfill` — render whatever got missed

Scans a directory and renders every `*.ndjson` that lacks a matching `*.summary.md`. Idempotent: a second run with no new logs renders nothing.

```sh
scribe backfill ~/.cache/ctrace/sessions/
scribe backfill --dry-run ~/.cache/ctrace/sessions/   # list the would-render set; write nothing
scribe backfill --force   ~/.cache/ctrace/sessions/   # re-render where the ndjson is newer than its summary
```

### `rollup` — a cross-session daily digest

Folds every `*.ndjson` in a time window through the shared parser and emits one aggregate digest: session count, top write-path prefixes, top binaries, outbound `connect()` by process, deletions, and flagged sensitive-path writes named to their source log. Memory stays bounded — only histograms and counters are kept, never the full path list — and the directory walk is internal, so there's no shell glob or `ARG_MAX` exposure.

```sh
scribe rollup --since today              # current calendar day (default)
scribe rollup --since 7d --top 30        # last 7 days, 30 entries per section
scribe rollup --since 24h --format json  # machine-readable
```

`--since` accepts `today`, `24h`, `48h`, `7d`, `30d`, or `Nd` / `Nh`.

### `render-session` — the hook-safe path

An idempotent single-session render meant for the SessionEnd hook: it writes the summary only if it's absent or would differ, and exits 0 either way — including when the existing summary is already byte-identical. `--timeout` bounds the work for a hook.

## How it works

One parser, one pass. Every subcommand streams the NDJSON through the same single-pass parser into bounded accumulators (histograms and counters), then a renderer turns those into Markdown or JSON. Streaming is the design constraint, not an optimization: the rollup has to handle hundreds of session logs in bounded memory, and the per-log render has to survive a truncated tail.

## Where it fits

The summarizing half of ctrace in the wintermute fleet. Sibling to `ctrace-orphan-reap`, which stops tracers orphaned by an ungraceful death and renders their logs through this same engine.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
