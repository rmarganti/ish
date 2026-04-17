mod cli;
mod config;
mod core;
mod model;
mod output;
mod roadmap;

use clap::Parser;
use std::process::ExitCode;

use crate::cli::prime_output;
use crate::config::{Config, find_config};
use crate::roadmap::{RoadmapOptions, roadmap_output};

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
    /// Generate a roadmap from milestone and epic hierarchy.
    Roadmap(RoadmapArgs),
    /// Print the current ish version.
    Version,
}

#[derive(clap::Args)]
struct RoadmapArgs {
    /// Include completed and scrapped items.
    #[arg(long)]
    include_done: bool,
    /// Output as JSON.
    #[arg(long)]
    json: bool,
    /// Filter milestones by status.
    #[arg(long = "status")]
    status: Vec<String>,
    /// Exclude milestones by status.
    #[arg(long = "no-status")]
    no_status: Vec<String>,
    /// Render plain IDs instead of markdown links.
    #[arg(long)]
    no_links: bool,
    /// Override the link prefix used in markdown links.
    #[arg(long)]
    link_prefix: Option<String>,
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
        Some(Commands::Roadmap(args)) => roadmap_command(args),
        Some(Commands::Version) => Ok(Some(version_output())),
        None => Ok(Some(
            "ish: no command specified. Run `ish --help` for usage.".to_string(),
        )),
    }
}

fn roadmap_command(args: RoadmapArgs) -> Result<Option<String>, String> {
    let current_dir = std::env::current_dir()
        .map_err(|error| format!("failed to determine current directory: {error}"))?;

    roadmap_output(
        &current_dir,
        &RoadmapOptions {
            include_done: args.include_done,
            status: args.status,
            no_status: args.no_status,
            no_links: args.no_links,
            link_prefix: args.link_prefix,
            json: args.json,
        },
    )
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
    use super::{Cli, Commands, RoadmapArgs, prime_command, roadmap_command, run, version_output};
    use crate::config::Config;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, MutexGuard, OnceLock};
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
        _lock: MutexGuard<'static, ()>,
        original: PathBuf,
    }

    impl WorkingDirGuard {
        fn change_to(path: &Path) -> Self {
            let lock = cwd_lock()
                .lock()
                .expect("working directory test lock should not be poisoned");
            let original = std::env::current_dir().expect("current directory should be readable");
            std::env::set_current_dir(path).expect("current directory should be changed");
            Self {
                _lock: lock,
                original,
            }
        }
    }

    impl Drop for WorkingDirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }

    fn cwd_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
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
            command: Some(Commands::Roadmap(RoadmapArgs {
                include_done: false,
                json: false,
                status: Vec::new(),
                no_status: Vec::new(),
                no_links: true,
                link_prefix: None,
            })),
        })
        .expect("roadmap command should succeed")
        .expect("roadmap command should print output");

        assert!(output.contains("# Roadmap"));
        assert!(output.contains("Milestone: Milestone (ish-m1)"));
    }

    #[test]
    fn roadmap_command_errors_without_config() {
        let temp = TestDir::new();
        let _guard = WorkingDirGuard::change_to(temp.path());

        let error = roadmap_command(RoadmapArgs {
            include_done: false,
            json: false,
            status: Vec::new(),
            no_status: Vec::new(),
            no_links: false,
            link_prefix: None,
        })
        .expect_err("roadmap command should fail without config");

        assert!(error.contains("no `.ish.yml` found"));
    }
}
