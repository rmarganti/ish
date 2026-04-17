use crate::core::store::{LinkCheckResult, LinkCycle, LinkRef, LinkType};
use crate::output::{danger, output_success, success, warning};
use serde_json::{Value, json};

use super::config::{ConfigChecks, link_issue_count};

pub(super) fn render_check_human(
    config_checks: &ConfigChecks,
    initial_links: &LinkCheckResult,
    final_links: &LinkCheckResult,
    fixed_links: Option<usize>,
) -> String {
    let mut lines = vec![format_check_line(
        config_checks.default_status.is_none(),
        "config default_status",
        config_checks.default_status.as_deref().unwrap_or("valid"),
    )];
    lines.push(format_check_line(
        config_checks.default_type.is_none(),
        "config default_type",
        config_checks.default_type.as_deref().unwrap_or("valid"),
    ));

    let color_details = if config_checks.invalid_colors.is_empty() {
        "all configured colors are supported".to_string()
    } else {
        config_checks.invalid_colors.join("; ")
    };
    lines.push(format_check_line(
        config_checks.invalid_colors.is_empty(),
        "config colors",
        &color_details,
    ));

    lines.push(format_check_line(
        initial_links.broken_links.is_empty(),
        "broken links",
        &link_ref_summary(&initial_links.broken_links),
    ));
    lines.push(format_check_line(
        initial_links.self_links.is_empty(),
        "self-references",
        &link_ref_summary(&initial_links.self_links),
    ));
    lines.push(format_check_line(
        initial_links.cycles.is_empty(),
        "cycles",
        &cycle_summary(&initial_links.cycles),
    ));

    if let Some(fixed_links) = fixed_links {
        let fix_message = if fixed_links == 0 {
            warning("No broken links or self-references needed fixing")
        } else {
            success(&format!("Applied --fix to {fixed_links} link(s)"))
        };
        lines.push(fix_message);
        lines.push(format_check_line(
            final_links.broken_links.is_empty(),
            "remaining broken links",
            &link_ref_summary(&final_links.broken_links),
        ));
        lines.push(format_check_line(
            final_links.self_links.is_empty(),
            "remaining self-references",
            &link_ref_summary(&final_links.self_links),
        ));
        lines.push(format_check_line(
            final_links.cycles.is_empty(),
            "remaining cycles",
            &cycle_summary(&final_links.cycles),
        ));
    }

    let issues_found = config_checks.issue_count() + link_issue_count(initial_links);
    let remaining = config_checks.issue_count() + link_issue_count(final_links);
    let fixed = fixed_links.unwrap_or(0);
    let summary = if issues_found == 0 {
        success("Summary: no issues found")
    } else if fixed_links.is_some() {
        warning(&format!(
            "Summary: {issues_found} issue(s) found, {fixed} fixed, {remaining} remaining"
        ))
    } else {
        warning(&format!("Summary: {issues_found} issue(s) found"))
    };
    lines.push(summary);

    lines.join("\n")
}

fn format_check_line(ok: bool, label: &str, details: &str) -> String {
    let icon = if ok { "✓" } else { "✗" };
    let text = format!("{icon} {label}: {details}");
    if ok { success(&text) } else { danger(&text) }
}

fn link_ref_summary(links: &[LinkRef]) -> String {
    if links.is_empty() {
        return "none".to_string();
    }

    links
        .iter()
        .map(|link| {
            format!(
                "{} {} {}",
                link.source_id,
                link_type_label(link.link_type),
                link.target_id
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn cycle_summary(cycles: &[LinkCycle]) -> String {
    if cycles.is_empty() {
        return "none".to_string();
    }

    cycles
        .iter()
        .map(|cycle| {
            format!(
                "{}: {}",
                link_type_label(cycle.link_type),
                cycle.path.join(" -> ")
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn link_type_label(link_type: LinkType) -> &'static str {
    match link_type {
        LinkType::Parent => "parent",
        LinkType::Blocking => "blocking",
        LinkType::BlockedBy => "blocked_by",
    }
}

pub(super) fn render_check_json(
    config_checks: &ConfigChecks,
    initial_links: &LinkCheckResult,
    final_links: &LinkCheckResult,
    fixed_links: Option<usize>,
) -> Result<String, String> {
    output_success(json!({
        "checks": {
            "config": {
                "default_status": {
                    "ok": config_checks.default_status.is_none(),
                    "message": config_checks.default_status.clone().unwrap_or_else(|| "valid".to_string()),
                },
                "default_type": {
                    "ok": config_checks.default_type.is_none(),
                    "message": config_checks.default_type.clone().unwrap_or_else(|| "valid".to_string()),
                },
                "colors": {
                    "ok": config_checks.invalid_colors.is_empty(),
                    "issues": config_checks.invalid_colors,
                }
            },
            "links": {
                "initial": link_checks_json(initial_links),
                "final": link_checks_json(final_links),
            }
        },
        "summary": {
            "issues_found": config_checks.issue_count() + link_issue_count(initial_links),
            "fixed_links": fixed_links.unwrap_or(0),
            "remaining_issues": config_checks.issue_count() + link_issue_count(final_links),
        }
    }))
}

fn link_checks_json(result: &LinkCheckResult) -> Value {
    json!({
        "broken_links": result.broken_links.iter().map(link_ref_json).collect::<Vec<_>>(),
        "self_links": result.self_links.iter().map(link_ref_json).collect::<Vec<_>>(),
        "cycles": result.cycles.iter().map(cycle_json).collect::<Vec<_>>(),
    })
}

fn link_ref_json(link: &LinkRef) -> Value {
    json!({
        "source_id": link.source_id,
        "link_type": link_type_label(link.link_type),
        "target_id": link.target_id,
    })
}

fn cycle_json(cycle: &LinkCycle) -> Value {
    json!({
        "link_type": link_type_label(cycle.link_type),
        "path": cycle.path,
    })
}
