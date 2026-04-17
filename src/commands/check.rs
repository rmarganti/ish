use crate::app::{AppContext, AppError, RunOutcome, json_output_error};
use crate::cli::CheckArgs;
use crate::config::Config;
use crate::core::store::{LinkCheckResult, LinkCycle, LinkRef, LinkType};
use crate::output::{ErrorCode, danger, is_supported_color_name, output_success, success, warning};
use serde_json::{Value, json};
use std::process::ExitCode;

pub(crate) fn check_command(args: CheckArgs, json: bool) -> Result<RunOutcome, AppError> {
    let context = AppContext::load()?;
    let config = context.config;
    let mut store = context.store;
    let config_checks = validate_config(&config);
    let initial_links = store.check_all_links();
    let issues_found = config_checks.issue_count() + link_issue_count(&initial_links);

    let fixed_links = if args.fix {
        Some(store.fix_broken_links().map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to fix broken links: {error}"),
            )
        })?)
    } else {
        None
    };
    let final_links = if args.fix {
        store.check_all_links()
    } else {
        initial_links.clone()
    };

    let output = if json {
        render_check_json(&config_checks, &initial_links, &final_links, fixed_links)
            .map_err(json_output_error)?
    } else {
        render_check_human(&config_checks, &initial_links, &final_links, fixed_links)
    };

    Ok(RunOutcome {
        output: Some(output),
        exit_code: if issues_found == 0 {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    })
}

#[derive(Debug, Clone)]
struct ConfigChecks {
    default_status: Option<String>,
    default_type: Option<String>,
    invalid_colors: Vec<String>,
}

impl ConfigChecks {
    fn issue_count(&self) -> usize {
        usize::from(self.default_status.is_some())
            + usize::from(self.default_type.is_some())
            + self.invalid_colors.len()
    }
}

fn validate_config(config: &Config) -> ConfigChecks {
    let mut invalid_colors = Vec::new();

    for status_name in config.status_names() {
        if let Some(status) = config.get_status(status_name)
            && !is_supported_color_name(status.color)
        {
            invalid_colors.push(format!(
                "status `{}` uses unsupported color `{}`",
                status.name, status.color
            ));
        }
    }

    for type_name in config.type_names() {
        if let Some(ishoo_type) = config.get_type(type_name)
            && !is_supported_color_name(ishoo_type.color)
        {
            invalid_colors.push(format!(
                "type `{}` uses unsupported color `{}`",
                ishoo_type.name, ishoo_type.color
            ));
        }
    }

    for priority_name in config.priority_names() {
        if let Some(priority) = config.get_priority(priority_name)
            && !is_supported_color_name(priority.color)
        {
            invalid_colors.push(format!(
                "priority `{}` uses unsupported color `{}`",
                priority.name, priority.color
            ));
        }
    }

    ConfigChecks {
        default_status: (!config.is_valid_status(&config.ish.default_status))
            .then(|| format!("invalid default_status `{}`", config.ish.default_status)),
        default_type: (!config.is_valid_type(&config.ish.default_type))
            .then(|| format!("invalid default_type `{}`", config.ish.default_type)),
        invalid_colors,
    }
}

fn link_issue_count(result: &LinkCheckResult) -> usize {
    result.broken_links.len() + result.self_links.len() + result.cycles.len()
}

fn render_check_human(
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

fn render_check_json(
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
