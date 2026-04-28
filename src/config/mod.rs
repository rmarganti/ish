#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const CONFIG_FILE_NAME: &str = ".ish.yml";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub ish: IshConfig,
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(skip)]
    statuses: Vec<StatusConfig>,
    #[serde(skip)]
    types: Vec<TypeConfig>,
    #[serde(skip)]
    priorities: Vec<PriorityConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct IshConfig {
    pub path: String,
    pub prefix: String,
    pub id_length: usize,
    pub default_status: String,
    pub default_type: String,
}

impl Default for IshConfig {
    fn default() -> Self {
        Config::default().ish
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ProjectConfig {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusConfig {
    pub name: &'static str,
    pub color: &'static str,
    pub archive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeConfig {
    pub name: &'static str,
    pub color: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorityConfig {
    pub name: &'static str,
    pub color: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        Self::default_with_prefix("ish")
    }
}

impl Config {
    pub fn default_with_prefix(prefix: impl Into<String>) -> Self {
        let prefix = prefix.into();
        let prefix = prefix.trim_end_matches('-').to_string();

        Self {
            ish: IshConfig {
                path: ".ish".to_string(),
                prefix,
                id_length: 4,
                default_status: "todo".to_string(),
                default_type: "task".to_string(),
            },
            project: ProjectConfig::default(),
            statuses: default_statuses(),
            types: default_types(),
            priorities: default_priorities(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let yaml = fs::read_to_string(path.as_ref()).map_err(ConfigError::Io)?;
        let mut config: Config = serde_yaml::from_str(&yaml).map_err(ConfigError::Yaml)?;
        config.populate_hardcoded_values();
        Ok(config)
    }

    pub fn save(&self, dir: impl AsRef<Path>) -> Result<PathBuf, ConfigError> {
        fs::create_dir_all(dir.as_ref()).map_err(ConfigError::Io)?;

        let path = dir.as_ref().join(CONFIG_FILE_NAME);
        let yaml = serde_yaml::to_string(self).map_err(ConfigError::Yaml)?;
        fs::write(&path, yaml).map_err(ConfigError::Io)?;

        Ok(path)
    }

    pub fn is_valid_status(&self, status: &str) -> bool {
        self.get_status(status).is_some()
    }

    pub fn is_valid_type(&self, ish_type: &str) -> bool {
        self.get_type(ish_type).is_some()
    }

    pub fn is_valid_priority(&self, priority: &str) -> bool {
        self.get_priority(priority).is_some()
    }

    pub fn status_names(&self) -> Vec<&'static str> {
        self.statuses.iter().map(|status| status.name).collect()
    }

    pub fn type_names(&self) -> Vec<&'static str> {
        self.types.iter().map(|ish_type| ish_type.name).collect()
    }

    pub fn priority_names(&self) -> Vec<&'static str> {
        self.priorities
            .iter()
            .map(|priority| priority.name)
            .collect()
    }

    pub fn is_archive_status(&self, status: &str) -> bool {
        self.get_status(status).is_some_and(|status| status.archive)
    }

    pub fn get_status(&self, name: &str) -> Option<&StatusConfig> {
        self.statuses.iter().find(|status| status.name == name)
    }

    pub fn get_type(&self, name: &str) -> Option<&TypeConfig> {
        self.types.iter().find(|ish_type| ish_type.name == name)
    }

    pub fn get_priority(&self, name: &str) -> Option<&PriorityConfig> {
        self.priorities
            .iter()
            .find(|priority| priority.name == name)
    }

    fn populate_hardcoded_values(&mut self) {
        self.statuses = default_statuses();
        self.types = default_types();
        self.priorities = default_priorities();
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Yaml(serde_yaml::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Yaml(error) => write!(f, "YAML error: {error}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Yaml(error) => Some(error),
        }
    }
}

pub fn find_config(start_dir: impl AsRef<Path>) -> Option<PathBuf> {
    let mut current = start_dir.as_ref();

    loop {
        let candidate = current.join(CONFIG_FILE_NAME);
        if candidate.is_file() {
            return Some(candidate);
        }

        current = current.parent()?;
    }
}

pub fn find_config_within(
    start_dir: impl AsRef<Path>,
    root_dir: impl AsRef<Path>,
) -> Option<PathBuf> {
    let root_dir = root_dir.as_ref();
    let mut current = start_dir.as_ref();

    if !current.starts_with(root_dir) {
        return None;
    }

    loop {
        let candidate = current.join(CONFIG_FILE_NAME);
        if candidate.is_file() {
            return Some(candidate);
        }

        if current == root_dir {
            return None;
        }

        current = current.parent()?;
    }
}

fn default_statuses() -> Vec<StatusConfig> {
    vec![
        StatusConfig {
            name: "in-progress",
            color: "yellow",
            archive: false,
        },
        StatusConfig {
            name: "todo",
            color: "green",
            archive: false,
        },
        StatusConfig {
            name: "draft",
            color: "blue",
            archive: false,
        },
        StatusConfig {
            name: "completed",
            color: "gray",
            archive: true,
        },
        StatusConfig {
            name: "scrapped",
            color: "gray",
            archive: true,
        },
    ]
}

fn default_types() -> Vec<TypeConfig> {
    vec![
        TypeConfig {
            name: "milestone",
            color: "cyan",
        },
        TypeConfig {
            name: "epic",
            color: "purple",
        },
        TypeConfig {
            name: "bug",
            color: "red",
        },
        TypeConfig {
            name: "feature",
            color: "green",
        },
        TypeConfig {
            name: "task",
            color: "blue",
        },
    ]
}

fn default_priorities() -> Vec<PriorityConfig> {
    vec![
        PriorityConfig {
            name: "critical",
            color: "red",
        },
        PriorityConfig {
            name: "high",
            color: "yellow",
        },
        PriorityConfig {
            name: "normal",
            color: "white",
        },
        PriorityConfig {
            name: "low",
            color: "gray",
        },
        PriorityConfig {
            name: "deferred",
            color: "gray",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{CONFIG_FILE_NAME, Config, find_config, find_config_within};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let unique = next_unique_suffix();
            let path = std::env::temp_dir().join(format!("ish-config-test-{unique}"));

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

    fn next_unique_suffix() -> String {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("{}-{}-{}", std::process::id(), timestamp, counter)
    }

    #[test]
    fn default_values_match_expected_defaults() {
        let config = Config::default();

        assert_eq!(config.ish.path, ".ish");
        assert_eq!(config.ish.prefix, "ish");
        assert_eq!(config.ish.id_length, 4);
        assert_eq!(config.ish.default_status, "todo");
        assert_eq!(config.ish.default_type, "task");
        assert_eq!(config.project.name, "");
        assert_eq!(
            config.status_names(),
            vec!["in-progress", "todo", "draft", "completed", "scrapped"]
        );
        assert_eq!(
            config.type_names(),
            vec!["milestone", "epic", "bug", "feature", "task"]
        );
        assert_eq!(
            config.priority_names(),
            vec!["critical", "high", "normal", "low", "deferred"]
        );
    }

    #[test]
    fn default_with_prefix_uses_custom_prefix() {
        let config = Config::default_with_prefix("bean");

        assert_eq!(config.ish.prefix, "bean");
        assert_eq!(config.ish.path, ".ish");
    }

    #[test]
    fn default_with_prefix_strips_trailing_dashes() {
        let config = Config::default_with_prefix("bean-");

        assert_eq!(config.ish.prefix, "bean");
    }

    #[test]
    fn save_and_load_round_trip_config() {
        let temp = TestDir::new();
        let mut config = Config::default_with_prefix("bean");
        config.ish.path = ".ish".to_string();
        config.ish.id_length = 6;
        config.ish.default_status = "draft".to_string();
        config.ish.default_type = "feature".to_string();
        config.project.name = "Issue Tracker".to_string();

        let saved_path = config.save(temp.path()).expect("config should save");
        let loaded = Config::load(&saved_path).expect("config should load");

        assert_eq!(saved_path, temp.path().join(CONFIG_FILE_NAME));
        assert_eq!(loaded, config);
        assert_eq!(
            loaded.get_status("completed").map(|status| status.color),
            Some("gray")
        );
        assert_eq!(
            loaded.get_type("epic").map(|ish_type| ish_type.color),
            Some("purple")
        );
        assert_eq!(
            loaded.get_priority("high").map(|priority| priority.color),
            Some("yellow")
        );
    }

    #[test]
    fn validation_helpers_cover_known_and_unknown_values() {
        let config = Config::default();

        assert!(config.is_valid_status("todo"));
        assert!(!config.is_valid_status("waiting"));
        assert!(config.is_valid_type("task"));
        assert!(!config.is_valid_type("chore"));
        assert!(config.is_valid_priority("high"));
        assert!(!config.is_valid_priority("urgent"));
        assert!(config.is_archive_status("completed"));
        assert!(config.is_archive_status("scrapped"));
        assert!(!config.is_archive_status("todo"));
        assert!(!config.is_archive_status("missing"));
    }

    #[test]
    fn load_populates_hardcoded_metadata_for_minimal_yaml() {
        let temp = TestDir::new();
        let config_path = temp.path().join(CONFIG_FILE_NAME);

        fs::write(
            &config_path,
            "ish:\n  path: .issues\n  prefix: proj\n  id_length: 5\n  default_status: draft\n  default_type: feature\nproject:\n  name: Demo\n",
        )
        .expect("config file should be written");

        let config = Config::load(&config_path).expect("config should load");

        assert_eq!(config.ish.path, ".issues");
        assert_eq!(config.ish.prefix, "proj");
        assert_eq!(config.project.name, "Demo");
        assert_eq!(
            config.get_status("in-progress").map(|status| status.color),
            Some("yellow")
        );
        assert_eq!(
            config.get_type("bug").map(|ish_type| ish_type.color),
            Some("red")
        );
        assert_eq!(
            config.get_priority("normal").map(|priority| priority.color),
            Some("white")
        );
    }

    #[test]
    fn finds_config_in_current_directory() {
        let temp = TestDir::new();
        let config = temp.path().join(CONFIG_FILE_NAME);

        fs::write(&config, "").expect("config file should be written");

        assert_eq!(find_config(temp.path()), Some(config));
    }

    #[test]
    fn finds_nearest_config_in_parent_directory() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("project");
        let nested_dir = project_dir.join("nested");
        let config = project_dir.join(CONFIG_FILE_NAME);

        fs::create_dir_all(&nested_dir).expect("nested dir should be created");
        fs::write(&config, "").expect("config file should be written");

        assert_eq!(find_config(&nested_dir), Some(config));
    }

    #[test]
    fn returns_none_when_no_config_exists() {
        let temp = TestDir::new();
        let nested_dir = temp.path().join("project").join("nested");

        fs::create_dir_all(&nested_dir).expect("nested dir should be created");

        assert_eq!(find_config(&nested_dir), None);
    }

    #[test]
    fn stops_search_at_root_directory() {
        let temp = TestDir::new();
        let root_dir = temp.path().join("workspace");
        let child_dir = root_dir.join("project").join("nested");
        let config_outside_root = temp.path().join(CONFIG_FILE_NAME);

        fs::create_dir_all(&child_dir).expect("child dir should be created");
        fs::write(&config_outside_root, "").expect("config file should be written");

        assert_eq!(find_config_within(&child_dir, &root_dir), None);
    }
}
