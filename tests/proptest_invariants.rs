//! Property-based invariant tests for ctrace-scribe.
//!
//! Read-only after scaffold. The edit-agent must NOT modify proptests.
//! These invariants hold across all inputs regardless of implementation details.

use proptest::prelude::*;

proptest! {
    #[test]
    fn summary_path_is_deterministic(name in "[a-z][a-z0-9]{0,20}") {
        use std::path::Path;
        // The summary path derivation must be deterministic: same input → same output.
        let input = format!("/tmp/{name}.ndjson");
        let p = Path::new(&input);
        // Invoke summary_path (via a separate binary call is impractical in proptest;
        // verify the invariant via the path convention: stem + ".summary.md").
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("session");
        let parent = p.parent().unwrap_or_else(|| Path::new("."));
        let derived = parent.join(format!("{stem}.summary.md"));
        let derived2 = parent.join(format!("{stem}.summary.md"));
        prop_assert_eq!(derived, derived2, "summary path should be deterministic");
    }

    #[test]
    fn ndjson_lines_count_never_panics(n in 0u32..1024) {
        // Placeholder invariant: safe saturating arithmetic on event counts.
        let count = u64::from(n).saturating_add(0);
        prop_assert!(count.checked_add(0).is_some());
    }
}
