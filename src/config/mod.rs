use std::path::{Path, PathBuf};

#[allow(dead_code)]
const CONFIG_FILE_NAME: &str = ".ish.yml";

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::{find_config, find_config_within};
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

    #[test]
    fn finds_config_in_current_directory() {
        let temp = TestDir::new();
        let config = temp.path().join(".ish.yml");

        fs::write(&config, "").expect("config file should be written");

        assert_eq!(find_config(temp.path()), Some(config));
    }

    #[test]
    fn finds_nearest_config_in_parent_directory() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("project");
        let nested_dir = project_dir.join("nested");
        let config = project_dir.join(".ish.yml");

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
        let config_outside_root = temp.path().join(".ish.yml");

        fs::create_dir_all(&child_dir).expect("child dir should be created");
        fs::write(&config_outside_root, "").expect("config file should be written");

        assert_eq!(find_config_within(&child_dir, &root_dir), None);
    }
}
