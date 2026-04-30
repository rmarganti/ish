use super::*;
use chrono::TimeZone;

fn sample_content() -> String {
    r#"---
# ish-a1b2
title: Fix the widget
status: todo
type: bug
priority: high
tags:
    - ui
    - regression
created_at: 2026-01-15T10:30:00Z
updated_at: 2026-01-16T14:00:00Z
parent: ish-p001
blocking:
    - ish-x001
blocked_by:
    - ish-y001
---

## Description

The widget is broken.

## Steps to Reproduce

1. Click the button
2. Observe the error
"#
    .to_string()
}

fn sample_ish() -> Ish {
    Ish {
            id: "ish-a1b2".to_string(),
            slug: "fix-the-widget".to_string(),
            path: "ish-a1b2--fix-the-widget.md".to_string(),
            title: "Fix the widget".to_string(),
            status: "todo".to_string(),
            ish_type: "bug".to_string(),
            priority: Some("high".to_string()),
            tags: vec!["ui".to_string(), "regression".to_string()],
            created_at: Utc.with_ymd_and_hms(2026, 1, 15, 10, 30, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 16, 14, 0, 0).unwrap(),
            order: None,
            body: "## Description\n\nThe widget is broken.\n\n## Steps to Reproduce\n\n1. Click the button\n2. Observe the error".to_string(),
            parent: Some("ish-p001".to_string()),
            blocking: vec!["ish-x001".to_string()],
            blocked_by: vec!["ish-y001".to_string()],
        }
}

#[test]
fn test_parse_basic() {
    let ish = Ish::parse("ish-a1b2--fix-the-widget.md", &sample_content()).expect("parse failed");

    assert_eq!(ish.id, "ish-a1b2");
    assert_eq!(ish.slug, "fix-the-widget");
    assert_eq!(ish.path, "ish-a1b2--fix-the-widget.md");
    assert_eq!(ish.title, "Fix the widget");
    assert_eq!(ish.status, "todo");
    assert_eq!(ish.ish_type, "bug");
    assert_eq!(ish.priority, Some("high".to_string()));
    assert_eq!(ish.tags, vec!["ui", "regression"]);
    assert_eq!(ish.parent, Some("ish-p001".to_string()));
    assert_eq!(ish.blocking, vec!["ish-x001"]);
    assert_eq!(ish.blocked_by, vec!["ish-y001"]);
    assert!(ish.body.contains("The widget is broken."));
    assert!(ish.body.contains("Steps to Reproduce"));
}

#[test]
fn test_parse_minimal() {
    let content = r#"---
# ish-min1
title: Minimal issue
status: todo
type: task
created_at: 2026-01-01T00:00:00Z
updated_at: 2026-01-01T00:00:00Z
---
"#;
    let ish = Ish::parse("ish-min1.md", content).expect("parse failed");

    assert_eq!(ish.id, "ish-min1");
    assert_eq!(ish.slug, "");
    assert_eq!(ish.title, "Minimal issue");
    assert_eq!(ish.priority, None);
    assert!(ish.tags.is_empty());
    assert!(ish.body.is_empty());
    assert!(ish.parent.is_none());
    assert!(ish.blocking.is_empty());
    assert!(ish.blocked_by.is_empty());
}

#[test]
fn test_render_round_trip() {
    let original = sample_ish();
    let rendered = original.render();
    let parsed =
        Ish::parse("ish-a1b2--fix-the-widget.md", &rendered).expect("round-trip parse failed");

    assert_eq!(original.id, parsed.id);
    assert_eq!(original.title, parsed.title);
    assert_eq!(original.status, parsed.status);
    assert_eq!(original.ish_type, parsed.ish_type);
    assert_eq!(original.priority, parsed.priority);
    assert_eq!(original.tags, parsed.tags);
    assert_eq!(original.parent, parsed.parent);
    assert_eq!(original.blocking, parsed.blocking);
    assert_eq!(original.blocked_by, parsed.blocked_by);
    assert_eq!(original.body, parsed.body);
}

#[test]
fn test_render_empty_body() {
    let mut ish = sample_ish();
    ish.body = String::new();

    let rendered = ish.render();
    assert!(rendered.ends_with("---\n"));

    let parsed = Ish::parse("ish-a1b2--fix-the-widget.md", &rendered).expect("parse failed");
    assert!(parsed.body.is_empty());
}

#[test]
fn test_render_minimal_fields() {
    let ish = Ish {
        id: "ish-x1".to_string(),
        slug: "simple".to_string(),
        path: "ish-x1--simple.md".to_string(),
        title: "Simple".to_string(),
        status: "todo".to_string(),
        ish_type: "task".to_string(),
        priority: None,
        tags: vec![],
        created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        order: None,
        body: String::new(),
        parent: None,
        blocking: vec![],
        blocked_by: vec![],
    };

    let rendered = ish.render();

    // Optional fields should not appear in the YAML.
    assert!(!rendered.contains("priority:"));
    assert!(!rendered.contains("tags:"));
    assert!(!rendered.contains("parent:"));
    assert!(!rendered.contains("blocking:"));
    assert!(!rendered.contains("blocked_by:"));
    assert!(!rendered.contains("order:"));
}

#[test]
fn test_parse_missing_frontmatter() {
    let content = "Just some markdown\n\nNo frontmatter here.";
    let result = Ish::parse("bad.md", content);
    assert!(matches!(result, Err(ParseError::MissingFrontmatter)));
}

#[test]
fn test_parse_missing_id() {
    let content = r#"---
title: No ID
status: todo
type: task
created_at: 2026-01-01T00:00:00Z
updated_at: 2026-01-01T00:00:00Z
---
"#;
    let result = Ish::parse("no-id.md", content);
    assert!(matches!(result, Err(ParseError::MissingId)));
}

#[test]
fn test_parse_filename_with_slug() {
    let (id, slug) = parse_filename("ish-a1b2--fix-the-widget.md");
    assert_eq!(id, "ish-a1b2");
    assert_eq!(slug, "fix-the-widget");
}

#[test]
fn test_parse_filename_without_slug() {
    let (id, slug) = parse_filename("ish-a1b2.md");
    assert_eq!(id, "ish-a1b2");
    assert_eq!(slug, "");
}

#[test]
fn test_parse_filename_uses_basename() {
    let (id, slug) = parse_filename(".ish/ish-a1b2--fix-the-widget.md");
    assert_eq!(id, "ish-a1b2");
    assert_eq!(slug, "fix-the-widget");
}

#[test]
fn test_build_filename_with_slug() {
    assert_eq!(
        build_filename("ish-a1b2", "fix-the-widget"),
        "ish-a1b2--fix-the-widget.md"
    );
}

#[test]
fn test_build_filename_without_slug() {
    assert_eq!(build_filename("ish-a1b2", ""), "ish-a1b2.md");
}

#[test]
fn test_filename_round_trip_with_slug() {
    let filename = build_filename("ish-a1b2", "fix-the-widget");
    let (id, slug) = parse_filename(&filename);

    assert_eq!(id, "ish-a1b2");
    assert_eq!(slug, "fix-the-widget");
}

#[test]
fn test_filename_round_trip_without_slug() {
    let filename = build_filename("ish-a1b2", "");
    let (id, slug) = parse_filename(&filename);

    assert_eq!(id, "ish-a1b2");
    assert_eq!(slug, "");
}

#[test]
fn test_slugify_normalizes_title() {
    assert_eq!(slugify("Fix the_widget"), "fix-the-widget");
}

#[test]
fn test_slugify_strips_unicode_and_special_characters() {
    assert_eq!(slugify("Crème brûlée!!!"), "crme-brle");
}

#[test]
fn test_slugify_collapses_repeated_separators() {
    assert_eq!(slugify("Hello___world -- again"), "hello-world-again");
}

#[test]
fn test_slugify_truncates_to_fifty_characters() {
    let slug = slugify("abcdefghijklmnopqrstuvwxyz abcdefghijklmnopqrstuvwxyz");

    assert_eq!(slug, "abcdefghijklmnopqrstuvwxyz-abcdefghijklmnopqrstuvw");
    assert_eq!(slug.len(), 50);
}

#[test]
fn test_new_id_uses_expected_format() {
    let id = new_id("ish", 6);

    assert!(id.starts_with("ish-"));
    assert_eq!(id.len(), 10);
    assert!(
        id[4..]
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
    );
}

#[test]
fn test_generated_ids_never_include_blocked_substrings() {
    for _ in 0..256 {
        let id = new_id("ish", 8);
        assert!(
            !contains_blocked_substring(&id),
            "generated blocked id: {id}"
        );
    }
}

#[test]
fn test_new_id_strips_trailing_dashes_from_prefix() {
    let id = new_id("ish-", 6);

    assert!(id.starts_with("ish-"));
    assert!(!id.starts_with("ish--"));
}

#[test]
fn test_contains_blocked_substring_detects_offensive_words() {
    assert!(contains_blocked_substring("ish-fuck"));
    assert!(!contains_blocked_substring("ish-safe1"));
}

#[test]
fn test_to_json_for_active_ish_includes_archived_false() {
    let ish = sample_ish();
    let json = ish.to_json("abc123");

    assert_eq!(json.id, "ish-a1b2");
    assert_eq!(json.etag, "abc123");
    assert_eq!(json.ish_type, "bug");
    assert!(!json.archived);
    assert!(!ish.is_archived());

    // Verify it serializes to JSON without error.
    let json_str = serde_json::to_string(&json).expect("JSON serialize failed");
    assert!(json_str.contains("\"type\":\"bug\""));
    assert!(json_str.contains("\"archived\":false"));
    assert!(json_str.contains("\"etag\":\"abc123\""));
}

#[test]
fn test_to_json_for_archived_ish_includes_archived_true() {
    let mut ish = sample_ish();
    ish.path = "archive/ish-a1b2--fix-the-widget.md".to_string();

    let json = ish.to_json("abc123");

    assert!(ish.is_archived());
    assert!(json.archived);
}

#[test]
fn test_etag_is_deterministic_for_same_content() {
    let first = sample_ish();
    let second = sample_ish();

    assert_eq!(first.etag(), second.etag());
}

#[test]
fn test_etag_changes_when_content_changes() {
    let first = sample_ish();
    let mut second = sample_ish();
    second.title = "Fix the other widget".to_string();

    assert_ne!(first.etag(), second.etag());
}

#[test]
fn test_etag_matches_expected_hash() {
    let ish = sample_ish();

    assert_eq!(ish.etag(), "bf9cff0b18dcadde");
}

#[test]
fn test_round_trip_parse_render_parse() {
    let content = sample_content();
    let first = Ish::parse("ish-a1b2--fix-the-widget.md", &content).expect("first parse");
    let rendered = first.render();
    let second = Ish::parse("ish-a1b2--fix-the-widget.md", &rendered).expect("second parse");

    assert_eq!(first, second);
}

#[test]
fn test_validate_tag_accepts_expected_format() {
    assert!(validate_tag("release-2026"));
}

#[test]
fn test_validate_tag_rejects_invalid_formats() {
    assert!(!validate_tag(""));
    assert!(!validate_tag("Release"));
    assert!(!validate_tag("1release"));
    assert!(!validate_tag("release--candidate"));
    assert!(!validate_tag("release-"));
    assert!(!validate_tag("release_candidate"));
}

#[test]
fn test_normalize_tag_trims_and_lowercases() {
    assert_eq!(normalize_tag("  Release-2026  "), "release-2026");
}

#[test]
fn test_has_tag_uses_normalized_lookup() {
    let ish = sample_ish();

    assert!(ish.has_tag(" UI "));
    assert!(!ish.has_tag("backend"));
}

#[test]
fn test_add_tag_normalizes_and_avoids_duplicates() {
    let mut ish = sample_ish();

    assert_eq!(ish.add_tag("  Backend  "), Ok(true));
    assert_eq!(ish.add_tag("backend"), Ok(false));
    assert_eq!(ish.tags, vec!["ui", "regression", "backend"]);
}

#[test]
fn test_add_tag_rejects_invalid_values() {
    let mut ish = sample_ish();

    assert_eq!(ish.add_tag("Not Valid"), Err(TagError::InvalidTag));
}

#[test]
fn test_remove_tag_uses_normalized_lookup() {
    let mut ish = sample_ish();

    assert!(ish.remove_tag(" REGRESSION "));
    assert!(!ish.remove_tag("backend"));
    assert_eq!(ish.tags, vec!["ui"]);
}

#[test]
fn test_replace_once_replaces_single_match() {
    assert_eq!(
        replace_once("before target after", "target", "updated"),
        Ok("before updated after".to_string())
    );
}

#[test]
fn test_replace_once_rejects_empty_needle() {
    assert_eq!(replace_once("body", "", "new"), Err(BodyError::EmptyNeedle));
}

#[test]
fn test_replace_once_rejects_missing_match() {
    assert_eq!(
        replace_once("body", "target", "new"),
        Err(BodyError::NotFound)
    );
}

#[test]
fn test_replace_once_rejects_multiple_matches() {
    assert_eq!(
        replace_once("target and target", "target", "new"),
        Err(BodyError::MultipleMatches)
    );
}

#[test]
fn test_unescape_body_handles_supported_sequences() {
    assert_eq!(
        unescape_body(r"line 1\nline 2\tindent\\path"),
        "line 1\nline 2\tindent\\path"
    );
}

#[test]
fn test_unescape_body_preserves_unknown_sequences() {
    assert_eq!(unescape_body(r"keep\xliteral\"), r"keep\xliteral\");
}

#[test]
fn test_append_with_separator_joins_non_empty_sections() {
    assert_eq!(append_with_separator("first", "second"), "first\n\nsecond");
}

#[test]
fn test_append_with_separator_returns_non_empty_side() {
    assert_eq!(append_with_separator("", "second"), "second");
    assert_eq!(append_with_separator("first", ""), "first");
}

#[test]
fn test_midpoint_uses_base62_middle_digit() {
    assert_eq!(midpoint(), "V");
}

#[test]
fn test_order_between_handles_empty_bounds() {
    assert_eq!(order_between("", ""), Some("V".to_string()));
    assert_eq!(order_between("", "V"), Some("F".to_string()));
    assert_eq!(order_between("V", ""), Some("VV".to_string()));
}

#[test]
fn test_order_between_returns_key_between_bounds() {
    let cases = [
        ("", "V"),
        ("V", ""),
        ("V", "W"),
        ("V", "k"),
        ("V", "VV"),
        ("F", "V"),
    ];

    for (lower, upper) in cases {
        let between = order_between(lower, upper).expect("valid bounds should produce a key");

        if !lower.is_empty() {
            assert!(
                lower < between.as_str(),
                "{lower} should sort before {between}"
            );
        }

        if !upper.is_empty() {
            assert!(
                between.as_str() < upper,
                "{between} should sort before {upper}"
            );
        }
    }
}

#[test]
fn test_increment_and_decrement_key_delegate_to_order_between() {
    assert_eq!(increment_key("V"), Some("VV".to_string()));
    assert_eq!(decrement_key("V"), Some("F".to_string()));
}

#[test]
fn test_order_between_repeated_insertions_stay_monotonic() {
    let mut lower = String::new();
    let mut generated = Vec::new();

    for _ in 0..8 {
        let next = order_between(&lower, "").expect("should generate a key after lower");
        generated.push(next.clone());
        lower = next;
    }

    assert!(generated.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn test_order_between_rejects_invalid_or_impossible_ranges() {
    assert_eq!(order_between("V", "V"), None);
    assert_eq!(order_between("V", "F"), None);
    assert_eq!(order_between("V", "V0"), None);
    assert_eq!(decrement_key("0"), None);
    assert_eq!(order_between("bad!", ""), None);
}
