use crate::model::ishoo::Ishoo;

/// Return ishoos whose title, slug, or body contains the query.
#[allow(dead_code)]
pub fn search<'a>(ishoos: &'a [Ishoo], query: &str) -> Vec<&'a Ishoo> {
    let needle = query.to_lowercase();

    ishoos
        .iter()
        .filter(|ishoo| {
            ishoo.title.to_lowercase().contains(&needle)
                || ishoo.slug.to_lowercase().contains(&needle)
                || ishoo.body.to_lowercase().contains(&needle)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::search;
    use crate::model::ishoo::Ishoo;
    use chrono::{TimeZone, Utc};

    fn sample_ishoo(id: &str, title: &str, slug: &str, body: &str) -> Ishoo {
        Ishoo {
            id: id.to_string(),
            slug: slug.to_string(),
            path: format!("{id}--{slug}.md"),
            title: title.to_string(),
            status: "todo".to_string(),
            ishoo_type: "task".to_string(),
            priority: None,
            tags: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            order: None,
            body: body.to_string(),
            parent: None,
            blocking: Vec::new(),
            blocked_by: Vec::new(),
        }
    }

    fn sample_ishoos() -> Vec<Ishoo> {
        vec![
            sample_ishoo(
                "ish-title",
                "Fix Widget Search",
                "fix-widget-search",
                "Track search regressions here.",
            ),
            sample_ishoo(
                "ish-body",
                "Add filter command",
                "add-filter-command",
                "Need a substring query for list output.",
            ),
            sample_ishoo(
                "ish-slug",
                "Polish terminal rendering",
                "search-rendering-tweaks",
                "No text match in the body.",
            ),
        ]
    }

    #[test]
    fn search_matches_title() {
        let ishoos = sample_ishoos();

        let matches = search(&ishoos, "widget");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "ish-title");
    }

    #[test]
    fn search_matches_body() {
        let ishoos = sample_ishoos();

        let matches = search(&ishoos, "substring query");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "ish-body");
    }

    #[test]
    fn search_matches_slug() {
        let ishoos = sample_ishoos();

        let matches = search(&ishoos, "rendering-tweaks");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "ish-slug");
    }

    #[test]
    fn search_is_case_insensitive() {
        let ishoos = sample_ishoos();

        let matches = search(&ishoos, "SeArCh");

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].id, "ish-title");
        assert_eq!(matches[1].id, "ish-slug");
    }

    #[test]
    fn search_returns_empty_when_nothing_matches() {
        let ishoos = sample_ishoos();

        let matches = search(&ishoos, "milestone");

        assert!(matches.is_empty());
    }
}
