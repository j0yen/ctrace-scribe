# ctrace-scribe

Single-pass ctrace NDJSON summary renderer with backfill support.

## Motivation

`ctrace` traces every Claude session to an NDJSON log and a SessionEnd hook renders a one-page Markdown summary — but only when the session exits gracefully. Heavy headless sessions are SIGKILLed by cgroup teardown before the hook runs, leaving logs permanently un-summarized. `ctrace-scribe` is the reusable engine that fixes this: a single-pass NDJSON→summary renderer plus `backfill` that renders every `*.ndjson` lacking a `*.summary.md`, idempotently.

## Usage

```
scribe render <log.ndjson> [--out PATH]
scribe backfill <dir> [--dry-run] [--force]
```

### `scribe render`

Renders a single ctrace NDJSON session log to `<log>.summary.md`. Use `--out -` to write to stdout.

```
scribe render ~/.cache/ctrace/sessions/claude-20260528T162617.ndjson
```

### `scribe backfill`

Scans a directory for every `*.ndjson` missing a `*.summary.md` and renders them. Idempotent: a second run with no new logs renders nothing.

```
scribe backfill ~/.cache/ctrace/sessions/
scribe backfill --dry-run ~/.cache/ctrace/sessions/   # list without rendering
scribe backfill --force ~/.cache/ctrace/sessions/     # re-render if ndjson is newer
```

## Acceptance criteria

1. `scribe render <log>` writes `<log>.summary.md` with Log: line, duration/event/PID/write count, and five section headers (Top binaries, Writes outside expected scope, Deletions, Outbound connect, Flagged when any flagged write exists).
2. Flagged section omitted when no sensitive-path writes; present and listing paths when `/home/jsy/.ssh/` write exists.
3. Renders 124k-event fixture in ≤ 2 s with a single file pass.
4. Truncated final line (ungraceful exit) does not fail the render; summary reports `N malformed lines skipped`.
5. `backfill` renders exactly the ndjson files missing a summary; leaves existing summaries untouched; prints `rendered N, skipped M`.
6. `backfill` run twice renders 0 on the second run (idempotent).
7. `backfill --dry-run` writes no files and exits 0, listing the would-render set.
8. `backfill --force` re-renders a log whose summary is older than the ndjson (mtime comparison).
9. `--help` documents `render` and `backfill` with their flags; exits 0.
10. Rendering a non-existent log exits non-zero with a usage error; an empty dir backfills 0 and exits 0.

## Installation

```
cargo install --path .
```

Or from the wintermute bootstrap installer:

```
curl -sSf https://raw.githubusercontent.com/j0yen/wintermute/main/bootstrap/install.sh | bash
```

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
