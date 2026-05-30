# Changelog

## v0.2.0 — 2026-05-30

`scribe rollup [--dir DIR] [--since WHEN] [--top N] [--format md|json]` — streams all `*.ndjson` in a directory/time window through the shared single-pass parser and emits a cross-session daily digest in Markdown or JSON format. Handles 800+ session logs in bounded memory with no ARG_MAX exposure. Sections: session count, top write-path prefixes, top binaries executed, outbound connect() by process, deletions, and flagged sensitive-path writes (with source log names).
