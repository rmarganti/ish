use crate::app::{AppError, json_output_error, load_config_from_current_dir};
use crate::cli::prime_output;
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

#[cfg(test)]
mod tests {
    use super::prime_command;
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
}
