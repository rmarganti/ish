use crate::app::{AppContext, AppError, classify_app_error, json_output_error};
use crate::cli::RoadmapArgs;
use crate::output::{ErrorCode, output_success};
use crate::roadmap::{RoadmapOptions, roadmap_output};
use serde_json::Value;

pub(crate) fn roadmap_command(args: RoadmapArgs, json: bool) -> Result<Option<String>, AppError> {
    let current_dir = AppContext::load()?.current_dir;

    let output = roadmap_output(
        &current_dir,
        &RoadmapOptions {
            include_done: args.include_done,
            status: args.status,
            no_status: args.no_status,
            no_links: args.no_links,
            link_prefix: args.link_prefix,
            json,
        },
    )
    .map_err(classify_app_error)?;

    if json {
        let Some(output) = output else {
            return Ok(None);
        };
        let data: Value = serde_json::from_str(&output).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to parse command JSON output: {error}"),
            )
        })?;
        Ok(Some(output_success(data).map_err(json_output_error)?))
    } else {
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::roadmap_command;
    use crate::app::run;
    use crate::cli::{Cli, Commands, RoadmapArgs};
    use crate::config::Config;
    use crate::test_support::{TestDir, WorkingDirGuard};
    use serde_json::Value;
    use std::fs;

    #[test]
    fn run_roadmap_returns_rendered_output_when_config_exists() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Roadmap Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-m1--milestone.md"),
            "---\n# ish-m1\ntitle: Milestone\nstatus: todo\ntype: milestone\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nMilestone body.\n",
        )
        .expect("milestone file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = run(Cli {
            command: Commands::Roadmap(RoadmapArgs {
                include_done: false,
                status: Vec::new(),
                no_status: Vec::new(),
                no_links: true,
                link_prefix: None,
            }),
            json: false,
        })
        .expect("roadmap command should succeed")
        .output
        .expect("roadmap command should print output");

        assert!(output.contains("# Roadmap"));
        assert!(output.contains("Milestone: Milestone (ish-m1)"));
    }

    #[test]
    fn roadmap_command_errors_without_config() {
        let temp = TestDir::new();
        let _guard = WorkingDirGuard::change_to(temp.path());

        let error = roadmap_command(
            RoadmapArgs {
                include_done: false,
                status: Vec::new(),
                no_status: Vec::new(),
                no_links: false,
                link_prefix: None,
            },
            false,
        )
        .expect_err("roadmap command should fail without config");

        assert!(error.message.contains("no `.ish.yml` found"));
    }

    #[test]
    fn run_roadmap_wraps_nested_json_in_response() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Roadmap JSON Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-m1--milestone.md"),
            "---\n# ish-m1\ntitle: Milestone\nstatus: todo\ntype: milestone\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nMilestone body.\n",
        )
        .expect("milestone file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = run(Cli {
            json: true,
            command: Commands::Roadmap(RoadmapArgs {
                include_done: false,
                status: Vec::new(),
                no_status: Vec::new(),
                no_links: true,
                link_prefix: None,
            }),
        })
        .expect("roadmap command should succeed")
        .output
        .expect("roadmap command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["milestones"][0]["milestone"]["id"], "ish-m1");
    }
}
