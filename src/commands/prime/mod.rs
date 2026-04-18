use crate::app::{AppError, json_output_error, load_config_from_current_dir};
use crate::config::Config;
use crate::output::output_message;

pub(crate) fn prime_command(json: bool) -> Result<Option<String>, AppError> {
    let Some((_, config)) = load_config_from_current_dir()? else {
        return Ok(None);
    };

    let output = prime_output(&config);
    if json {
        Ok(Some(output_message(output).map_err(json_output_error)?))
    } else {
        Ok(Some(output))
    }
}

const PRIME_PROMPT_TEMPLATE: &str = include_str!("prime_prompt.tmpl");

const COMMAND_REFERENCE: [(&str, &str); 11] = [
    (
        "init",
        "Create a new `.ish.yml` and initialize the ish workspace.",
    ),
    (
        "create",
        "Create a new ish markdown file with YAML frontmatter.",
    ),
    (
        "list",
        "List ishes, optionally filtered by status, type, or text search.",
    ),
    ("update", "Update ish metadata or body content in place."),
    (
        "show",
        "Show full ish details, including rendered markdown body output.",
    ),
    ("delete", "Delete an ish markdown file."),
    (
        "archive",
        "Move completed or scrapped ishes out of the active set.",
    ),
    (
        "check",
        "Run validation checks against the current ish workspace.",
    ),
    (
        "roadmap",
        "Show hierarchy and dependency views across related ishes.",
    ),
    (
        "prime",
        "Print this AI-agent guide for the current project.",
    ),
    ("version", "Print the current ish version."),
];

fn prime_output(config: &Config) -> String {
    let commands = format_commands();
    let types = format_types(config);
    let statuses = format_statuses(config);
    let priorities = format_priorities(config);

    render_template(
        PRIME_PROMPT_TEMPLATE,
        &[
            ("project_name", project_name(config)),
            ("config_path", &config.ish.path),
            ("prefix", &config.ish.prefix),
            ("default_type", &config.ish.default_type),
            ("default_status", &config.ish.default_status),
            ("commands", &commands),
            ("types", &types),
            ("statuses", &statuses),
            ("priorities", &priorities),
        ],
    )
}

fn project_name(config: &Config) -> &str {
    let trimmed = config.project.name.trim();
    if trimmed.is_empty() {
        "this project"
    } else {
        trimmed
    }
}

fn format_commands() -> String {
    COMMAND_REFERENCE
        .iter()
        .map(|(command, description)| format!("- `ish {command}`: {description}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_types(config: &Config) -> String {
    config
        .type_names()
        .into_iter()
        .map(|name| {
            let color = config
                .get_type(name)
                .map(|ish_type| ish_type.color)
                .unwrap_or("unknown");
            format!("- `{name}` (color: `{color}`)")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_statuses(config: &Config) -> String {
    config
        .status_names()
        .into_iter()
        .map(|name| {
            let status = config.get_status(name);
            let color = status.map(|status| status.color).unwrap_or("unknown");
            let archive_note = if config.is_archive_status(name) {
                "; archive status"
            } else {
                ""
            };
            format!("- `{name}` (color: `{color}`{archive_note})")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_priorities(config: &Config) -> String {
    config
        .priority_names()
        .into_iter()
        .map(|name| {
            let color = config
                .get_priority(name)
                .map(|priority| priority.color)
                .unwrap_or("unknown");
            format!("- `{name}` (color: `{color}`)")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_template(template: &str, replacements: &[(&str, &str)]) -> String {
    let mut rendered = template.to_string();

    for (key, value) in replacements {
        rendered = rendered.replace(&format!("{{{{{key}}}}}"), value);
    }

    rendered
}

#[cfg(test)]
mod tests {
    use super::prime_command;
    use super::prime_output;
    use crate::app::run;
    use crate::cli::{Cli, Commands};
    use crate::config::Config;
    use crate::test_support::{TestDir, WorkingDirGuard};
    use serde_json::Value;

    #[test]
    fn run_prime_returns_rendered_guide_when_config_exists() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Prime Test".to_string();
        config.save(temp.path()).expect("config should save");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = run(Cli {
            json: false,
            command: Some(Commands::Prime),
        })
        .expect("prime command should succeed")
        .output
        .expect("prime command should print output");

        assert!(output.contains("# ish Agent Guide"));
        assert!(output.contains("Prime Test"));
    }

    #[test]
    fn prime_command_silently_exits_without_config() {
        let temp = TestDir::new();
        let _guard = WorkingDirGuard::change_to(temp.path());

        assert!(
            prime_command(false)
                .expect("prime command should succeed")
                .is_none()
        );
    }

    #[test]
    fn prime_command_wraps_markdown_in_json_mode() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Prime JSON Test".to_string();
        config.save(temp.path()).expect("config should save");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = prime_command(true)
            .expect("prime command should succeed")
            .expect("prime command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert!(
            parsed["message"]
                .as_str()
                .expect("message should be present")
                .contains("# ish Agent Guide")
        );
    }

    #[test]
    fn prime_output_uses_config_values() {
        let mut config = Config::default_with_prefix("proj");
        config.project.name = "Ish Tracker".to_string();
        config.ish.path = ".custom-ish".to_string();
        config.ish.default_type = "feature".to_string();
        config.ish.default_status = "draft".to_string();

        let output = prime_output(&config);

        assert!(output.contains("# ish Agent Guide"));
        assert!(output.contains("in Ish Tracker"));
        assert!(
            output.contains("- `ish prime`: Print this AI-agent guide for the current project.")
        );
        assert!(output.contains("- `ish update`: Update ish metadata or body content in place."));
        assert!(output.contains(
            "- `ish show`: Show full ish details, including rendered markdown body output."
        ));
        assert!(output.contains("- `feature` (color: `green`)"));
        assert!(output.contains("- `completed` (color: `gray`; archive status)"));
        assert!(output.contains("- `critical` (color: `red`)"));
        assert!(output.contains("Workspace path: `.custom-ish`"));
        assert!(output.contains("Default new ish type: `feature`"));
        assert!(output.contains("Default new ish status: `draft`"));
        assert!(output.contains("Prefix new IDs with `proj-`"));
        assert!(output.contains("Use `ish` as the source of truth for work in Ish Tracker."));
        assert!(output.contains("Use `ish list --json` for machine-readable output."));
        assert!(!output.contains("{{commands}}"));
    }

    #[test]
    fn prime_output_falls_back_to_generic_project_name() {
        let output = prime_output(&Config::default());

        assert!(output.contains("in this project"));
    }
}
