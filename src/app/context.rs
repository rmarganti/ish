use crate::app::error::{AppError, store_open_error};
use crate::config::{Config, find_config};
use crate::core::store::Store;
use crate::output::ErrorCode;
use std::path::{Path, PathBuf};

pub struct AppContext {
    pub current_dir: PathBuf,
    pub config: Config,
    pub store: Store,
}

impl AppContext {
    pub fn load() -> Result<Self, AppError> {
        let current_dir = current_dir()?;
        let config_path = require_config_path(&current_dir)?;
        let config = load_config(&config_path)?;
        let store_root = store_root(&config_path, &config)?;
        let mut store =
            Store::new(&store_root, config.clone()).map_err(store_open_error(&store_root))?;
        store.load().map_err(store_open_error(&store_root))?;

        Ok(Self {
            current_dir,
            config,
            store,
        })
    }
}

pub fn current_dir() -> Result<PathBuf, AppError> {
    std::env::current_dir().map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to determine current directory: {error}"),
        )
    })
}

pub fn find_config_path() -> Result<Option<PathBuf>, AppError> {
    Ok(find_config(current_dir()?))
}

pub fn load_config_from_current_dir() -> Result<Option<(PathBuf, Config)>, AppError> {
    let Some(config_path) = find_config_path()? else {
        return Ok(None);
    };

    let config = load_config(&config_path)?;
    Ok(Some((config_path, config)))
}

fn require_config_path(current_dir: &Path) -> Result<PathBuf, AppError> {
    find_config(current_dir).ok_or_else(|| {
        AppError::new(
            ErrorCode::NotFound,
            "no `.ish.yml` found in the current directory or its parents",
        )
    })
}

fn load_config(config_path: &Path) -> Result<Config, AppError> {
    Config::load(config_path).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to load `{}`: {error}", config_path.display()),
        )
    })
}

fn store_root(config_path: &Path, config: &Config) -> Result<PathBuf, AppError> {
    Ok(config_path
        .parent()
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::FileError,
                format!("invalid config path: {}", config_path.display()),
            )
        })?
        .join(&config.ish.path))
}
