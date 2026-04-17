mod cli;
mod config;
mod core;
mod model;
mod output;

use clap::Parser;
use std::process::ExitCode;

use crate::cli::prime_output;
use crate::config::{Config, find_config};

/// A terminal-based issue tracker.
#[derive(Parser)]
#[command(name = "ish", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Print AI-agent guidance for the current ish project.
    Prime,
    /// Print the current ish version.
    Version,
}

fn version_output() -> String {
    format!("ish {}", env!("CARGO_PKG_VERSION"))
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(Some(output)) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Ok(None) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ish: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<Option<String>, String> {
    match cli.command {
        Some(Commands::Prime) => prime_command(),
        Some(Commands::Version) => Ok(Some(version_output())),
        None => Ok(Some(
            "ish: no command specified. Run `ish --help` for usage.".to_string(),
        )),
    }
}

fn prime_command() -> Result<Option<String>, String> {
    let current_dir = std::env::current_dir()
        .map_err(|error| format!("failed to determine current directory: {error}"))?;
    let Some(config_path) = find_config(&current_dir) else {
        return Ok(None);
    };

    let config = Config::load(&config_path)
        .map_err(|error| format!("failed to load `{}`: {error}", config_path.display()))?;

    Ok(Some(prime_output(&config)))
}

#[cfg(test)]
mod tests {
    use super::{Cli, Commands, prime_command, run, version_output};
    use crate::config::Config;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("ish-main-test-{unique}"));
            fs::create_dir_all(&path).expect("temp dir should be created");

            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    struct WorkingDirGuard {
        original: PathBuf,
    }

    impl WorkingDirGuard {
        fn change_to(path: &Path) -> Self {
            let original = std::env::current_dir().expect("current directory should be readable");
            std::env::set_current_dir(path).expect("current directory should be changed");
            Self { original }
        }
    }

    impl Drop for WorkingDirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }

    #[test]
    fn version_output_uses_package_version() {
        assert_eq!(
            version_output(),
            format!("ish {}", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn run_prime_returns_rendered_guide_when_config_exists() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Prime Test".to_string();
        config.save(temp.path()).expect("config should save");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = run(Cli {
            command: Some(Commands::Prime),
        })
        .expect("prime command should succeed")
        .expect("prime command should print output");

        assert!(output.contains("# ish Agent Guide"));
        assert!(output.contains("Prime Test"));
    }

    #[test]
    fn prime_command_silently_exits_without_config() {
        let temp = TestDir::new();
        let _guard = WorkingDirGuard::change_to(temp.path());

        assert!(
            prime_command()
                .expect("prime command should succeed")
                .is_none()
        );
    }
}
