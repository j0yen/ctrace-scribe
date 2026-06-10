# Changelog

## v0.3.0 — 2026-06-10

`scribe render-session <log.ndjson> [--timeout <secs>]` — idempotent, hook-safe
single-session render. Returns exit 0 whether it writes a new summary or finds
the existing one already byte-identical (no-op). Replaces the inline render call
in the `ctrace-session-end.sh` SessionEnd hook; the hook now falls back to
`ctrace status --format json` for the log path when `claude-owns.json` is absent
or stale, and reports genuine failures to docket rather than silently swallowing
them. `sigpipe::reset()` added to main so piped invocations never panic.

## v0.2.0 — 2026-05-30

`scribe rollup [--dir DIR] [--since WHEN] [--top N] [--format md|json]` — streams all `*.ndjson` in a directory/time window through the shared single-pass parser and emits a cross-session daily digest in Markdown or JSON format. Handles 800+ session logs in bounded memory with no ARG_MAX exposure. Sections: session count, top write-path prefixes, top binaries executed, outbound connect() by process, deletions, and flagged sensitive-path writes (with source log names).
