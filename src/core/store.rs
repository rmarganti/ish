use crate::config::Config;
use crate::model::ish::{
    BodyError, Ish, append_with_separator, build_filename, new_id, replace_once, slugify,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

const ARCHIVE_DIR_NAME: &str = "archive";

#[derive(Debug)]
pub struct Store {
    root: PathBuf,
    config: Config,
    ishes: HashMap<String, Ish>,
}

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    InvalidPath(PathBuf),
    InvalidFrontmatter(PathBuf),
    InvalidStatus(String),
    InvalidType(String),
    InvalidPriority(String),
    InvalidTag(String),
    ParentNotAllowed(String),
    InvalidParentType {
        child_type: String,
        parent_id: String,
        parent_type: String,
        allowed_parent_types: Vec<&'static str>,
    },
    NotFound(String),
    ETagMismatch {
        expected: String,
        actual: String,
    },
    Body(BodyError),
    Yaml {
        path: PathBuf,
        source: serde_yaml::Error,
    },
}

#[derive(Debug, Clone, Default)]
pub struct CreateIsh {
    pub title: String,
    pub status: Option<String>,
    pub ish_type: Option<String>,
    pub priority: Option<String>,
    pub body: String,
    pub tags: Vec<String>,
    pub parent: Option<String>,
    pub blocking: Vec<String>,
    pub blocked_by: Vec<String>,
    pub id_prefix: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateIsh {
    pub status: Option<String>,
    pub ish_type: Option<String>,
    pub priority: Option<Option<String>>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub body_replace: Option<(String, String)>,
    pub body_append: Option<String>,
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
    pub parent: Option<Option<String>>,
    pub add_blocking: Vec<String>,
    pub remove_blocking: Vec<String>,
    pub add_blocked_by: Vec<String>,
    pub remove_blocked_by: Vec<String>,
    pub if_match: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct DiskFrontmatter {
    #[serde(default)]
    title: String,
    #[serde(default)]
    status: String,
    #[serde(rename = "type", default)]
    ish_type: String,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    order: Option<String>,
    #[serde(default)]
    parent: Option<String>,
    #[serde(default)]
    blocking: Vec<String>,
    #[serde(default)]
    blocked_by: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkType {
    Parent,
    Blocking,
    BlockedBy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRef {
    pub source_id: String,
    pub link_type: LinkType,
    pub target_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkCycle {
    pub link_type: LinkType,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinkCheckResult {
    pub broken_links: Vec<LinkRef>,
    pub self_links: Vec<LinkRef>,
    pub cycles: Vec<LinkCycle>,
}

impl Store {
    pub fn new(root: impl Into<PathBuf>, config: Config) -> Result<Self, StoreError> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(StoreError::Io)?;

        Ok(Self {
            root,
            config,
            ishes: HashMap::new(),
        })
    }

    pub fn load(&mut self) -> Result<(), StoreError> {
        self.ishes.clear();
        let root = self.root.clone();
        self.load_dir(&root)
    }

    pub fn load_one(&self, id: &str) -> Result<Ish, StoreError> {
        let normalized_id = self.normalize_id(id);
        let path = self
            .find_ish_path(&self.root, &normalized_id)?
            .ok_or_else(|| StoreError::NotFound(normalized_id.clone()))?;
        self.load_ish(&path)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn all(&self) -> Vec<&Ish> {
        self.ishes.values().collect()
    }

    pub fn get(&self, id: &str) -> Option<&Ish> {
        self.ishes
            .get(id)
            .or_else(|| self.ishes.get(&self.normalize_id(id)))
    }

    pub fn create(&mut self, input: CreateIsh) -> Result<Ish, StoreError> {
        let status = input
            .status
            .unwrap_or_else(|| self.config.ish.default_status.clone());
        let ish_type = input
            .ish_type
            .unwrap_or_else(|| self.config.ish.default_type.clone());
        let priority = input.priority.or_else(|| Some("normal".to_string()));

        self.validate_status(&status)?;
        self.validate_type(&ish_type)?;
        if let Some(priority_name) = priority.as_deref() {
            self.validate_priority(priority_name)?;
        }

        let id = self.generate_unique_id(input.id_prefix.as_deref());
        let slug = slugify(&input.title);
        let path = build_filename(&id, &slug);
        let now = Utc::now();
        let mut ish = Ish {
            id: id.clone(),
            slug,
            path,
            title: input.title,
            status,
            ish_type,
            priority,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            order: None,
            body: input.body,
            parent: input.parent.map(|parent| self.normalize_id(&parent)),
            blocking: normalize_ids(self, input.blocking),
            blocked_by: normalize_ids(self, input.blocked_by),
        };

        for tag in input.tags {
            ish.add_tag(&tag)
                .map_err(|_| StoreError::InvalidTag(tag.clone()))?;
        }

        if let Some(parent_id) = ish.parent.as_deref() {
            self.validate_parent(&ish, parent_id)?;
        }

        self.save_to_disk(&ish)?;
        self.ishes.insert(id, ish.clone());
        Ok(ish)
    }

    pub fn update(&mut self, id: &str, changes: UpdateIsh) -> Result<Ish, StoreError> {
        let normalized_id = self.normalize_id(id);
        let current = self
            .ishes
            .get(&normalized_id)
            .ok_or_else(|| StoreError::NotFound(normalized_id.clone()))?
            .clone();

        if let Some(if_match) = changes.if_match.as_deref() {
            let actual = current.etag();
            if if_match != actual {
                return Err(StoreError::ETagMismatch {
                    expected: if_match.to_string(),
                    actual,
                });
            }
        }

        let mut updated = current.clone();

        if let Some(status) = changes.status {
            self.validate_status(&status)?;
            updated.status = status;
        }

        if let Some(ish_type) = changes.ish_type {
            self.validate_type(&ish_type)?;
            updated.ish_type = ish_type;
        }

        if let Some(priority) = changes.priority {
            if let Some(priority_name) = priority.as_deref() {
                self.validate_priority(priority_name)?;
            }
            updated.priority = priority;
        }

        if let Some(title) = changes.title {
            updated.title = title;
            updated.slug = slugify(&updated.title);
            updated.path = build_filename(&updated.id, &updated.slug);
        }

        if let Some(body) = changes.body {
            updated.body = body;
        }

        if let Some((old, new)) = changes.body_replace {
            updated.body = replace_once(&updated.body, &old, &new).map_err(StoreError::Body)?;
        }

        if let Some(addition) = changes.body_append {
            updated.body = append_with_separator(&updated.body, &addition);
        }

        for tag in changes.add_tags {
            updated
                .add_tag(&tag)
                .map_err(|_| StoreError::InvalidTag(tag.clone()))?;
        }

        for tag in changes.remove_tags {
            updated.remove_tag(&tag);
        }

        if let Some(parent) = changes.parent {
            updated.parent = parent.map(|parent| self.normalize_id(&parent));
        }

        update_relation_list(
            self,
            &mut updated.blocking,
            changes.add_blocking,
            changes.remove_blocking,
        );
        update_relation_list(
            self,
            &mut updated.blocked_by,
            changes.add_blocked_by,
            changes.remove_blocked_by,
        );

        if let Some(parent_id) = updated.parent.clone() {
            self.validate_parent(&updated, &parent_id)?;
        }

        updated.updated_at = Utc::now();

        let original_path = self.root.join(&current.path);
        let updated_path = self.root.join(&updated.path);
        if current.path != updated.path && original_path.exists() {
            fs::rename(&original_path, &updated_path).map_err(StoreError::Io)?;
        }

        self.save_to_disk(&updated)?;
        self.ishes.insert(normalized_id, updated.clone());
        Ok(updated)
    }

    pub fn delete(&mut self, id: &str) -> Result<Ish, StoreError> {
        let normalized_id = self.normalize_id(id);
        let removed = self
            .ishes
            .remove(&normalized_id)
            .ok_or_else(|| StoreError::NotFound(normalized_id.clone()))?;

        let path = self.root.join(&removed.path);
        fs::remove_file(&path).map_err(StoreError::Io)?;

        let mut dirty_ids = Vec::new();
        for (other_id, other) in &mut self.ishes {
            let mut dirty = false;

            if other.parent.as_deref() == Some(normalized_id.as_str()) {
                other.parent = None;
                dirty = true;
            }

            dirty |= retain_without(&mut other.blocking, &normalized_id);
            dirty |= retain_without(&mut other.blocked_by, &normalized_id);

            if dirty {
                other.updated_at = Utc::now();
                dirty_ids.push(other_id.clone());
            }
        }

        for dirty_id in dirty_ids {
            let ish = self
                .ishes
                .get(&dirty_id)
                .ok_or_else(|| StoreError::NotFound(dirty_id.clone()))?
                .clone();
            self.save_to_disk(&ish)?;
        }

        Ok(removed)
    }

    pub fn save_to_disk(&self, ish: &Ish) -> Result<(), StoreError> {
        let path = self.root.join(&ish.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(StoreError::Io)?;
        }
        fs::write(path, ish.render()).map_err(StoreError::Io)
    }

    pub fn normalize_id(&self, id: &str) -> String {
        let prefix = self.config.ish.prefix.trim_end_matches('-');

        if prefix.is_empty() || id == prefix || id.starts_with(&format!("{prefix}-")) {
            id.to_string()
        } else {
            format!("{prefix}-{id}")
        }
    }

    pub fn archive(&mut self, id: &str) -> Result<(), StoreError> {
        let normalized_id = self.normalize_id(id);
        let source_path = self.ish_absolute_path(&normalized_id)?;
        let file_name = source_path
            .file_name()
            .ok_or_else(|| StoreError::InvalidPath(source_path.clone()))?;
        let archive_dir = self.root.join(ARCHIVE_DIR_NAME);
        fs::create_dir_all(&archive_dir).map_err(StoreError::Io)?;
        let destination_path = archive_dir.join(file_name);

        fs::rename(&source_path, &destination_path).map_err(StoreError::Io)?;

        let relative_path = self.relative_path(&destination_path)?;
        self.ishes
            .get_mut(&normalized_id)
            .ok_or_else(|| StoreError::NotFound(normalized_id.clone()))?
            .path = relative_path;
        Ok(())
    }

    pub fn unarchive(&mut self, id: &str) -> Result<(), StoreError> {
        let normalized_id = self.normalize_id(id);
        let source_path = self.ish_absolute_path(&normalized_id)?;
        let file_name = source_path
            .file_name()
            .ok_or_else(|| StoreError::InvalidPath(source_path.clone()))?;
        let destination_path = self.root.join(file_name);

        fs::rename(&source_path, &destination_path).map_err(StoreError::Io)?;

        let relative_path = self.relative_path(&destination_path)?;
        self.ishes
            .get_mut(&normalized_id)
            .ok_or_else(|| StoreError::NotFound(normalized_id.clone()))?
            .path = relative_path;
        Ok(())
    }

    pub fn is_archived(&self, id: &str) -> Result<bool, StoreError> {
        let normalized_id = self.normalize_id(id);
        let ish = self
            .ishes
            .get(&normalized_id)
            .ok_or(StoreError::NotFound(normalized_id))?;
        Ok(ish.path.starts_with(&format!("{ARCHIVE_DIR_NAME}/")))
    }

    pub fn load_and_unarchive(&mut self, id: &str) -> Result<(), StoreError> {
        let normalized_id = self.normalize_id(id);

        if self.ishes.contains_key(&normalized_id) {
            return self.unarchive(&normalized_id);
        }

        let archive_path = self.find_archived_path(&normalized_id)?;
        let mut ish = self.load_ish(&archive_path)?;
        let destination_path = self.root.join(
            archive_path
                .file_name()
                .ok_or_else(|| StoreError::InvalidPath(archive_path.clone()))?,
        );

        fs::rename(&archive_path, &destination_path).map_err(StoreError::Io)?;

        ish.path = self.relative_path(&destination_path)?;
        self.ishes.insert(ish.id.clone(), ish);
        Ok(())
    }

    pub fn archive_all_completed(&mut self) -> Result<usize, StoreError> {
        let ids_to_archive = self
            .ishes
            .values()
            .filter(|ish| {
                self.config.is_archive_status(&ish.status)
                    && !ish.path.starts_with(&format!("{ARCHIVE_DIR_NAME}/"))
            })
            .map(|ish| ish.id.clone())
            .collect::<Vec<_>>();

        for id in &ids_to_archive {
            self.archive(id)?;
        }

        Ok(ids_to_archive.len())
    }

    pub fn detect_cycle(&self, from_id: &str, link_type: LinkType, to_id: &str) -> bool {
        self.find_cycle_path(from_id, link_type, to_id).is_some()
    }

    pub fn find_incoming_links(&self, target_id: &str) -> Vec<LinkRef> {
        let target_id = self.normalize_id(target_id);
        let mut incoming = Vec::new();

        for ish in self.ishes.values() {
            if ish.parent.as_deref() == Some(target_id.as_str()) {
                incoming.push(LinkRef {
                    source_id: ish.id.clone(),
                    link_type: LinkType::Parent,
                    target_id: target_id.clone(),
                });
            }

            for linked_id in &ish.blocking {
                if linked_id == &target_id {
                    incoming.push(LinkRef {
                        source_id: ish.id.clone(),
                        link_type: LinkType::Blocking,
                        target_id: target_id.clone(),
                    });
                }
            }

            for linked_id in &ish.blocked_by {
                if linked_id == &target_id {
                    incoming.push(LinkRef {
                        source_id: ish.id.clone(),
                        link_type: LinkType::BlockedBy,
                        target_id: target_id.clone(),
                    });
                }
            }
        }

        incoming.sort_by(|left, right| {
            left.source_id
                .cmp(&right.source_id)
                .then_with(|| link_type_rank(left.link_type).cmp(&link_type_rank(right.link_type)))
                .then_with(|| left.target_id.cmp(&right.target_id))
        });
        incoming
    }

    pub fn find_active_blockers(&self, id: &str) -> Vec<String> {
        let normalized_id = self.normalize_id(id);
        let mut blockers = HashSet::new();

        let Some(ish) = self.ishes.get(&normalized_id) else {
            return Vec::new();
        };

        for blocker_id in &ish.blocked_by {
            if self.has_active_status(blocker_id) {
                blockers.insert(blocker_id.clone());
            }
        }

        for candidate in self.ishes.values() {
            if candidate
                .blocking
                .iter()
                .any(|blocked_id| blocked_id == &normalized_id)
                && !self.config.is_archive_status(&candidate.status)
            {
                blockers.insert(candidate.id.clone());
            }
        }

        let mut blockers = blockers.into_iter().collect::<Vec<_>>();
        blockers.sort();
        blockers
    }

    pub fn find_all_blockers(&self, id: &str) -> Vec<String> {
        let mut blockers = self
            .find_active_blockers(id)
            .into_iter()
            .collect::<HashSet<_>>();

        self.walk_parent_chain(id, |ancestor| {
            blockers.extend(self.find_active_blockers(&ancestor.id));
            true
        });

        let mut blockers = blockers.into_iter().collect::<Vec<_>>();
        blockers.sort();
        blockers
    }

    pub fn is_blocked(&self, id: &str) -> bool {
        !self.find_all_blockers(id).is_empty()
    }

    pub fn is_explicitly_blocked(&self, id: &str) -> bool {
        !self.find_active_blockers(id).is_empty()
    }

    pub fn is_implicitly_blocked(&self, id: &str) -> bool {
        let mut is_blocked = false;

        self.walk_parent_chain(id, |ancestor| {
            if !self.find_active_blockers(&ancestor.id).is_empty() {
                is_blocked = true;
                return false;
            }

            true
        });

        is_blocked
    }

    pub fn implicit_status(&self, id: &str) -> Option<(String, String)> {
        let mut inherited = None;

        self.walk_parent_chain(id, |ancestor| {
            if self.config.is_archive_status(&ancestor.status) {
                inherited = Some((ancestor.status.clone(), ancestor.id.clone()));
                return false;
            }

            true
        });

        inherited
    }

    pub fn check_all_links(&self) -> LinkCheckResult {
        let mut result = LinkCheckResult::default();
        let mut seen_cycles = HashSet::new();

        for ish in self.ishes.values() {
            for link in collect_links(ish) {
                if link.source_id == link.target_id {
                    result.self_links.push(link.clone());
                    continue;
                }

                if !self.ishes.contains_key(&link.target_id) {
                    result.broken_links.push(link.clone());
                    continue;
                }

                if let Some(path) =
                    self.find_cycle_path(&link.source_id, link.link_type, &link.target_id)
                {
                    let path = canonical_cycle_path(&path);
                    let key = cycle_key(link.link_type, &path);
                    if seen_cycles.insert(key) {
                        result.cycles.push(LinkCycle {
                            link_type: link.link_type,
                            path,
                        });
                    }
                }
            }
        }

        result.broken_links.sort_by(link_ref_cmp);
        result.self_links.sort_by(link_ref_cmp);
        result.cycles.sort_by(|left, right| {
            link_type_rank(left.link_type)
                .cmp(&link_type_rank(right.link_type))
                .then_with(|| left.path.cmp(&right.path))
        });

        result
    }

    pub fn fix_broken_links(&mut self) -> Result<usize, StoreError> {
        let existing_ids = self.ishes.keys().cloned().collect::<HashSet<_>>();
        let mut dirty_ids = Vec::new();
        let mut fixed_count = 0;

        for (id, ish) in &mut self.ishes {
            let mut dirty = false;

            if let Some(parent) = ish.parent.as_deref()
                && (parent == id.as_str() || !existing_ids.contains(parent))
            {
                ish.parent = None;
                dirty = true;
                fixed_count += 1;
            }

            let blocking_before = ish.blocking.len();
            ish.blocking
                .retain(|target| target != id && existing_ids.contains(target));
            let removed_blocking = blocking_before - ish.blocking.len();
            if removed_blocking > 0 {
                dirty = true;
                fixed_count += removed_blocking;
            }

            let blocked_by_before = ish.blocked_by.len();
            ish.blocked_by
                .retain(|target| target != id && existing_ids.contains(target));
            let removed_blocked_by = blocked_by_before - ish.blocked_by.len();
            if removed_blocked_by > 0 {
                dirty = true;
                fixed_count += removed_blocked_by;
            }

            if dirty {
                ish.updated_at = Utc::now();
                dirty_ids.push(id.clone());
            }
        }

        for dirty_id in dirty_ids {
            let ish = self
                .ishes
                .get(&dirty_id)
                .ok_or_else(|| StoreError::NotFound(dirty_id.clone()))?
                .clone();
            self.save_to_disk(&ish)?;
        }

        Ok(fixed_count)
    }

    fn load_dir(&mut self, dir: &Path) -> Result<(), StoreError> {
        for entry in fs::read_dir(dir).map_err(StoreError::Io)? {
            let entry = entry.map_err(StoreError::Io)?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(StoreError::Io)?;

            if file_type.is_dir() {
                if path != self.root && is_hidden(&path) {
                    continue;
                }

                self.load_dir(&path)?;
                continue;
            }

            if file_type.is_file() && path.extension() == Some(OsStr::new("md")) {
                let ish = self.load_ish(&path)?;
                self.ishes.insert(ish.id.clone(), ish);
            }
        }

        Ok(())
    }

    fn load_ish(&self, path: &Path) -> Result<Ish, StoreError> {
        let content = fs::read_to_string(path).map_err(StoreError::Io)?;
        let metadata = fs::metadata(path).map_err(StoreError::Io)?;
        let modified_at = metadata.modified().map_err(StoreError::Io)?;
        let modified_at = DateTime::<Utc>::from(modified_at);
        let (_, yaml, body) = split_frontmatter(&content)
            .ok_or_else(|| StoreError::InvalidFrontmatter(path.to_path_buf()))?;
        let fm: DiskFrontmatter =
            serde_yaml::from_str(&yaml).map_err(|source| StoreError::Yaml {
                path: path.to_path_buf(),
                source,
            })?;
        let relative_path = path
            .strip_prefix(&self.root)
            .map_err(|_| StoreError::InvalidPath(path.to_path_buf()))?;
        let relative_path = relative_path.to_string_lossy().replace('\\', "/");
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| StoreError::InvalidPath(path.to_path_buf()))?;
        let (id, slug) = crate::model::ish::parse_filename(filename);
        let created_at = fm.created_at.unwrap_or(modified_at);
        let updated_at = fm.updated_at.unwrap_or(created_at);

        Ok(Ish {
            id,
            slug,
            path: relative_path,
            title: fm.title,
            status: default_string(fm.status, &self.config.ish.default_status),
            ish_type: default_string(fm.ish_type, &self.config.ish.default_type),
            priority: Some(default_optional_string(fm.priority, "normal")),
            tags: fm.tags,
            created_at,
            updated_at,
            order: fm.order,
            body,
            parent: fm.parent,
            blocking: fm.blocking,
            blocked_by: fm.blocked_by,
        })
    }

    fn find_ish_path(&self, dir: &Path, id: &str) -> Result<Option<PathBuf>, StoreError> {
        for entry in fs::read_dir(dir).map_err(StoreError::Io)? {
            let entry = entry.map_err(StoreError::Io)?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(StoreError::Io)?;

            if file_type.is_dir() {
                if path != self.root && is_hidden(&path) {
                    continue;
                }

                if let Some(found) = self.find_ish_path(&path, id)? {
                    return Ok(Some(found));
                }

                continue;
            }

            if !file_type.is_file() || path.extension() != Some(OsStr::new("md")) {
                continue;
            }

            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| StoreError::InvalidPath(path.clone()))?;
            let (candidate_id, _) = crate::model::ish::parse_filename(file_name);

            if candidate_id == id {
                return Ok(Some(path));
            }
        }

        Ok(None)
    }

    fn ish_absolute_path(&self, id: &str) -> Result<PathBuf, StoreError> {
        let ish = self
            .ishes
            .get(id)
            .ok_or_else(|| StoreError::NotFound(id.to_string()))?;
        Ok(self.root.join(&ish.path))
    }

    fn relative_path(&self, path: &Path) -> Result<String, StoreError> {
        let relative_path = path
            .strip_prefix(&self.root)
            .map_err(|_| StoreError::InvalidPath(path.to_path_buf()))?;
        Ok(relative_path.to_string_lossy().replace('\\', "/"))
    }

    fn find_archived_path(&self, id: &str) -> Result<PathBuf, StoreError> {
        let archive_dir = self.root.join(ARCHIVE_DIR_NAME);
        if !archive_dir.is_dir() {
            return Err(StoreError::NotFound(id.to_string()));
        }

        for entry in fs::read_dir(&archive_dir).map_err(StoreError::Io)? {
            let entry = entry.map_err(StoreError::Io)?;
            let path = entry.path();
            if path.extension() != Some(OsStr::new("md")) {
                continue;
            }

            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| StoreError::InvalidPath(path.clone()))?;
            let (candidate_id, _) = crate::model::ish::parse_filename(file_name);

            if candidate_id == id {
                return Ok(path);
            }
        }

        Err(StoreError::NotFound(id.to_string()))
    }

    fn generate_unique_id(&self, prefix_override: Option<&str>) -> String {
        let prefix = prefix_override.unwrap_or(&self.config.ish.prefix);

        loop {
            let id = new_id(prefix, self.config.ish.id_length);
            if !self.ishes.contains_key(&id) {
                return id;
            }
        }
    }

    fn validate_status(&self, status: &str) -> Result<(), StoreError> {
        if self.config.is_valid_status(status) {
            Ok(())
        } else {
            Err(StoreError::InvalidStatus(status.to_string()))
        }
    }

    fn validate_type(&self, ish_type: &str) -> Result<(), StoreError> {
        if self.config.is_valid_type(ish_type) {
            Ok(())
        } else {
            Err(StoreError::InvalidType(ish_type.to_string()))
        }
    }

    fn validate_priority(&self, priority: &str) -> Result<(), StoreError> {
        if self.config.is_valid_priority(priority) {
            Ok(())
        } else {
            Err(StoreError::InvalidPriority(priority.to_string()))
        }
    }

    fn valid_parent_types(ish_type: &str) -> &'static [&'static str] {
        match ish_type {
            "milestone" => &[],
            "epic" => &["milestone"],
            "feature" => &["milestone", "epic"],
            "task" | "bug" => &["milestone", "epic", "feature"],
            _ => &[],
        }
    }

    fn validate_parent(&self, ish: &Ish, parent_id: &str) -> Result<(), StoreError> {
        let allowed_parent_types = Self::valid_parent_types(&ish.ish_type);
        if allowed_parent_types.is_empty() {
            return Err(StoreError::ParentNotAllowed(ish.ish_type.clone()));
        }

        let normalized_parent_id = self.normalize_id(parent_id);
        let parent = self
            .ishes
            .get(&normalized_parent_id)
            .ok_or_else(|| StoreError::NotFound(normalized_parent_id.clone()))?;

        if allowed_parent_types.contains(&parent.ish_type.as_str()) {
            Ok(())
        } else {
            Err(StoreError::InvalidParentType {
                child_type: ish.ish_type.clone(),
                parent_id: normalized_parent_id,
                parent_type: parent.ish_type.clone(),
                allowed_parent_types: allowed_parent_types.to_vec(),
            })
        }
    }

    fn find_cycle_path(
        &self,
        from_id: &str,
        link_type: LinkType,
        to_id: &str,
    ) -> Option<Vec<String>> {
        let from_id = self.normalize_id(from_id);
        let to_id = self.normalize_id(to_id);

        if from_id == to_id {
            return Some(vec![from_id.clone(), from_id]);
        }

        let mut stack = vec![(to_id.clone(), vec![from_id.clone(), to_id.clone()])];
        let mut visited = HashSet::from([to_id]);

        while let Some((current_id, path)) = stack.pop() {
            for next_id in self.link_targets(&current_id, link_type) {
                if next_id == from_id {
                    let mut cycle = path.clone();
                    cycle.push(from_id.clone());
                    return Some(cycle);
                }

                if visited.insert(next_id.clone()) {
                    let mut next_path = path.clone();
                    next_path.push(next_id.clone());
                    stack.push((next_id, next_path));
                }
            }
        }

        None
    }

    fn link_targets(&self, id: &str, link_type: LinkType) -> Vec<String> {
        let Some(ish) = self.ishes.get(id) else {
            return Vec::new();
        };

        match link_type {
            LinkType::Parent => ish.parent.iter().cloned().collect(),
            LinkType::Blocking => ish.blocking.clone(),
            LinkType::BlockedBy => ish.blocked_by.clone(),
        }
    }

    fn walk_parent_chain<F>(&self, id: &str, mut visitor: F)
    where
        F: FnMut(&Ish) -> bool,
    {
        let mut next_parent = self
            .ishes
            .get(&self.normalize_id(id))
            .and_then(|ish| ish.parent.clone());
        let mut visited = HashSet::new();

        while let Some(parent_id) = next_parent {
            if !visited.insert(parent_id.clone()) {
                break;
            }

            let Some(parent) = self.ishes.get(&parent_id) else {
                break;
            };

            if !visitor(parent) {
                break;
            }

            next_parent = parent.parent.clone();
        }
    }

    fn has_active_status(&self, id: &str) -> bool {
        self.ishes
            .get(id)
            .is_some_and(|ish| !self.config.is_archive_status(&ish.status))
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(error) => write!(f, "I/O error: {error}"),
            StoreError::InvalidPath(path) => write!(f, "invalid ish path: {}", path.display()),
            StoreError::InvalidFrontmatter(path) => {
                write!(f, "invalid frontmatter in `{}`", path.display())
            }
            StoreError::InvalidStatus(status) => write!(f, "invalid status: {status}"),
            StoreError::InvalidType(ish_type) => write!(f, "invalid type: {ish_type}"),
            StoreError::InvalidPriority(priority) => write!(f, "invalid priority: {priority}"),
            StoreError::InvalidTag(tag) => write!(f, "invalid tag: {tag}"),
            StoreError::ParentNotAllowed(ish_type) => {
                write!(f, "type `{ish_type}` cannot have a parent")
            }
            StoreError::InvalidParentType {
                child_type,
                parent_id,
                parent_type,
                allowed_parent_types,
            } => write!(
                f,
                "invalid parent `{parent_id}` of type `{parent_type}` for child type `{child_type}`; allowed parent types: {}",
                allowed_parent_types.join(", ")
            ),
            StoreError::NotFound(id) => write!(f, "ish not found: {id}"),
            StoreError::ETagMismatch { expected, actual } => {
                write!(f, "etag mismatch: expected {expected}, actual {actual}")
            }
            StoreError::Body(error) => write!(f, "body update failed: {error}"),
            StoreError::Yaml { path, source } => {
                write!(f, "failed to parse YAML in `{}`: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StoreError::Io(error) => Some(error),
            StoreError::Body(error) => Some(error),
            StoreError::Yaml { source, .. } => Some(source),
            StoreError::InvalidPath(_)
            | StoreError::InvalidFrontmatter(_)
            | StoreError::InvalidStatus(_)
            | StoreError::InvalidType(_)
            | StoreError::InvalidPriority(_)
            | StoreError::InvalidTag(_)
            | StoreError::ParentNotAllowed(_)
            | StoreError::InvalidParentType { .. }
            | StoreError::NotFound(_)
            | StoreError::ETagMismatch { .. } => None,
        }
    }
}

fn normalize_ids(store: &Store, ids: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();

    for id in ids {
        let id = store.normalize_id(&id);
        if !normalized.iter().any(|existing| existing == &id) {
            normalized.push(id);
        }
    }

    normalized
}

fn update_relation_list(
    store: &Store,
    values: &mut Vec<String>,
    additions: Vec<String>,
    removals: Vec<String>,
) {
    for id in additions {
        let normalized = store.normalize_id(&id);
        if !values.iter().any(|existing| existing == &normalized) {
            values.push(normalized);
        }
    }

    for id in removals {
        let normalized = store.normalize_id(&id);
        values.retain(|existing| existing != &normalized);
    }
}

fn retain_without(values: &mut Vec<String>, removed_id: &str) -> bool {
    let original_len = values.len();
    values.retain(|value| value != removed_id);
    values.len() != original_len
}

fn collect_links(ish: &Ish) -> Vec<LinkRef> {
    let mut links = Vec::new();

    if let Some(parent) = &ish.parent {
        links.push(LinkRef {
            source_id: ish.id.clone(),
            link_type: LinkType::Parent,
            target_id: parent.clone(),
        });
    }

    for target_id in &ish.blocking {
        links.push(LinkRef {
            source_id: ish.id.clone(),
            link_type: LinkType::Blocking,
            target_id: target_id.clone(),
        });
    }

    for target_id in &ish.blocked_by {
        links.push(LinkRef {
            source_id: ish.id.clone(),
            link_type: LinkType::BlockedBy,
            target_id: target_id.clone(),
        });
    }

    links
}

fn cycle_key(link_type: LinkType, path: &[String]) -> String {
    let nodes = canonical_cycle_nodes(path);
    format!("{}:{}", link_type_rank(link_type), nodes.join("->"))
}

fn canonical_cycle_path(path: &[String]) -> Vec<String> {
    let mut nodes = canonical_cycle_nodes(path);
    if let Some(first) = nodes.first().cloned() {
        nodes.push(first);
    }
    nodes
}

fn canonical_cycle_nodes(path: &[String]) -> Vec<String> {
    let nodes = &path[..path.len().saturating_sub(1)];
    if nodes.is_empty() {
        return Vec::new();
    }

    let mut best = nodes.to_vec();
    for start in 1..nodes.len() {
        let rotated = nodes[start..]
            .iter()
            .chain(nodes[..start].iter())
            .cloned()
            .collect::<Vec<_>>();
        if rotated < best {
            best = rotated;
        }
    }

    best
}

fn link_type_rank(link_type: LinkType) -> usize {
    match link_type {
        LinkType::Parent => 0,
        LinkType::Blocking => 1,
        LinkType::BlockedBy => 2,
    }
}

fn link_ref_cmp(left: &LinkRef, right: &LinkRef) -> std::cmp::Ordering {
    left.source_id
        .cmp(&right.source_id)
        .then_with(|| link_type_rank(left.link_type).cmp(&link_type_rank(right.link_type)))
        .then_with(|| left.target_id.cmp(&right.target_id))
}

fn default_string(value: String, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn default_optional_string(value: Option<String>, fallback: &str) -> String {
    match value {
        Some(value) if !value.trim().is_empty() => value,
        _ => fallback.to_string(),
    }
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

fn split_frontmatter(content: &str) -> Option<(String, String, String)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    let after_open = trimmed[3..].strip_prefix('\n').unwrap_or(&trimmed[3..]);
    let close_pos = after_open.find("\n---")?;
    let frontmatter_block = &after_open[..close_pos];
    let after_close = &after_open[close_pos + 4..];

    let mut yaml_lines = Vec::new();
    let mut id = None;

    for line in frontmatter_block.lines() {
        if id.is_none() && line.starts_with("# ") {
            id = Some(line[2..].trim().to_string());
        } else {
            yaml_lines.push(line);
        }
    }

    Some((id?, yaml_lines.join("\n"), after_close.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        CreateIsh, LinkCheckResult, LinkCycle, LinkRef, LinkType, Store, StoreError, UpdateIsh,
    };
    use crate::config::Config;
    use chrono::{TimeZone, Utc};
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
            let path = std::env::temp_dir().join(format!("ish-store-test-{unique}"));
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
    fn new_initializes_root_directory() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");

        let _store = Store::new(&root, Config::default()).expect("store should initialize");

        assert!(root.is_dir());
        assert!(!root.join(".gitignore").exists());
    }

    #[test]
    fn load_reads_markdown_files_including_archive_and_skips_hidden_dirs() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        let archive_dir = root.join("archive");
        let hidden_dir = root.join(".hidden");

        fs::create_dir_all(&archive_dir).expect("archive dir should exist");
        fs::create_dir_all(&hidden_dir).expect("hidden dir should exist");
        write_ish(
            &root.join("ish-abcd--top-level.md"),
            "ish-abcd",
            "Top Level",
            "todo",
            "task",
            Some("normal"),
            "Top level body.",
        );
        write_ish(
            &archive_dir.join("ish-efgh--archived.md"),
            "ish-efgh",
            "Archived",
            "completed",
            "task",
            Some("low"),
            "Archived body.",
        );
        write_ish(
            &hidden_dir.join("ish-skip--hidden.md"),
            "ish-skip",
            "Hidden",
            "todo",
            "task",
            Some("normal"),
            "Hidden body.",
        );

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        let mut ids = store
            .all()
            .into_iter()
            .map(|ish| ish.id.as_str())
            .collect::<Vec<_>>();
        ids.sort_unstable();

        assert_eq!(ids, vec!["ish-abcd", "ish-efgh"]);
        assert_eq!(
            store
                .get("ish-efgh")
                .expect("archived ish should load")
                .path,
            "archive/ish-efgh--archived.md"
        );
    }

    #[test]
    fn load_applies_defaults_for_empty_fields_and_uses_filename_metadata() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        let path = root.join("ish-abcd--needs-defaults.md");
        fs::create_dir_all(&root).expect("root dir should exist");
        fs::write(
            &path,
            "---\n# ignored-frontmatter-id\ntitle: Needs defaults\nstatus: \ntype: \npriority: \ntags: []\nupdated_at: 2026-01-02T03:04:05Z\n---\n\nBody text.\n",
        )
        .expect("ish file should be written");

        let mut config = Config::default_with_prefix("ish");
        config.ish.default_status = "todo".to_string();
        config.ish.default_type = "task".to_string();

        let mut store = Store::new(&root, config).expect("store should initialize");
        store.load().expect("store should load files");

        let ish = store.get("abcd").expect("normalized id should resolve");

        assert_eq!(ish.id, "ish-abcd");
        assert_eq!(ish.slug, "needs-defaults");
        assert_eq!(ish.path, "ish-abcd--needs-defaults.md");
        assert_eq!(ish.status, "todo");
        assert_eq!(ish.ish_type, "task");
        assert_eq!(ish.priority.as_deref(), Some("normal"));
        assert!(ish.tags.is_empty());
        assert!(ish.blocking.is_empty());
        assert_eq!(
            ish.updated_at,
            Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap()
        );
        assert!(ish.created_at <= Utc::now());
    }

    #[test]
    fn load_one_reads_a_single_ish_by_full_or_short_id() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        let archive_dir = root.join("archive");

        fs::create_dir_all(&archive_dir).expect("archive dir should exist");
        write_ish(
            &archive_dir.join("ish-abcd--archived.md"),
            "ish-abcd",
            "Archived",
            "completed",
            "task",
            Some("normal"),
            "Archived body.",
        );

        let store = Store::new(&root, Config::default()).expect("store should initialize");

        let by_short_id = store.load_one("abcd").expect("short id should resolve");
        let by_full_id = store
            .load_one("ish-abcd")
            .expect("full id should resolve as well");

        assert_eq!(by_short_id.id, "ish-abcd");
        assert_eq!(by_short_id.path, "archive/ish-abcd--archived.md");
        assert_eq!(by_short_id.body, "Archived body.");
        assert_eq!(by_full_id.id, by_short_id.id);
        assert_eq!(by_full_id.path, by_short_id.path);
    }

    #[test]
    fn load_one_returns_not_found_for_unknown_id() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("root dir should exist");

        let store = Store::new(&root, Config::default()).expect("store should initialize");
        let error = store
            .load_one("missing")
            .expect_err("unknown ids should return not found");

        assert!(matches!(error, StoreError::NotFound(ref id) if id == "ish-missing"));
    }

    #[test]
    fn load_one_returns_parse_errors_for_corrupted_files() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("root dir should exist");
        fs::write(
            root.join("ish-bad1--corrupted.md"),
            "---\n# ish-bad1\ntitle: Corrupted\nstatus: todo\ntype: [\n---\n\nBody.\n",
        )
        .expect("corrupted ish file should be written");

        let store = Store::new(&root, Config::default()).expect("store should initialize");
        let error = store
            .load_one("bad1")
            .expect_err("corrupted files should surface parse errors");

        assert!(matches!(error, StoreError::Yaml { .. }));
    }

    #[test]
    fn normalize_id_preserves_existing_prefix() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        let store =
            Store::new(&root, Config::default_with_prefix("ish")).expect("store should initialize");

        assert_eq!(store.normalize_id("abcd"), "ish-abcd");
        assert_eq!(store.normalize_id("ish-abcd"), "ish-abcd");
    }

    #[test]
    fn normalize_id_strips_trailing_dashes_from_prefix() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        let mut config = Config::default_with_prefix("ish");
        config.ish.prefix = "ish-".to_string();
        let store = Store::new(&root, config).expect("store should initialize");

        assert_eq!(store.normalize_id("abcd"), "ish-abcd");
        assert_eq!(store.normalize_id("ish-abcd"), "ish-abcd");
    }

    #[test]
    fn normalize_id_requires_a_prefix_boundary() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        let store =
            Store::new(&root, Config::default_with_prefix("ish")).expect("store should initialize");

        assert_eq!(store.normalize_id("ishx-abcd"), "ish-ishx-abcd");
    }

    #[test]
    fn archive_and_unarchive_move_files_and_update_store_paths() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        let active_path = root.join("ish-abcd--active.md");
        let archived_path = root.join("archive/ish-abcd--active.md");

        fs::create_dir_all(&root).expect("root dir should exist");
        write_ish(
            &active_path,
            "ish-abcd",
            "Active",
            "todo",
            "task",
            Some("normal"),
            "Body.",
        );

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        store.archive("abcd").expect("archive should succeed");

        assert!(!active_path.exists());
        assert!(archived_path.exists());
        assert!(store.is_archived("ish-abcd").expect("ish should exist"));
        assert_eq!(
            store.get("ish-abcd").expect("ish should exist").path,
            "archive/ish-abcd--active.md"
        );

        store
            .unarchive("ish-abcd")
            .expect("unarchive should succeed");

        assert!(active_path.exists());
        assert!(!archived_path.exists());
        assert!(!store.is_archived("abcd").expect("ish should exist"));
        assert_eq!(
            store.get("ish-abcd").expect("ish should exist").path,
            "ish-abcd--active.md"
        );
    }

    #[test]
    fn load_and_unarchive_restores_archived_file_into_store() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        let archive_dir = root.join("archive");
        let archived_path = archive_dir.join("ish-abcd--active.md");
        let active_path = root.join("ish-abcd--active.md");

        fs::create_dir_all(&archive_dir).expect("archive dir should exist");
        write_ish(
            &archived_path,
            "ish-abcd",
            "Active",
            "completed",
            "task",
            Some("normal"),
            "Body.",
        );

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");

        store
            .load_and_unarchive("abcd")
            .expect("load and unarchive should succeed");

        assert!(active_path.exists());
        assert!(!archived_path.exists());
        assert_eq!(
            store.get("ish-abcd").expect("ish should be loaded").path,
            "ish-abcd--active.md"
        );
    }

    #[test]
    fn archive_all_completed_moves_only_archive_statuses() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");

        fs::create_dir_all(&root).expect("root dir should exist");
        write_ish(
            &root.join("ish-todo--active.md"),
            "ish-todo",
            "Todo",
            "todo",
            "task",
            Some("normal"),
            "Todo body.",
        );
        write_ish(
            &root.join("ish-done--completed.md"),
            "ish-done",
            "Done",
            "completed",
            "task",
            Some("normal"),
            "Done body.",
        );
        write_ish(
            &root.join("ish-nope--scrapped.md"),
            "ish-nope",
            "Nope",
            "scrapped",
            "task",
            Some("normal"),
            "Nope body.",
        );

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        let archived_count = store
            .archive_all_completed()
            .expect("bulk archive should succeed");

        assert_eq!(archived_count, 2);
        assert!(root.join("ish-todo--active.md").exists());
        assert!(root.join("archive/ish-done--completed.md").exists());
        assert!(root.join("archive/ish-nope--scrapped.md").exists());
        assert_eq!(
            store.get("ish-done").expect("done ish should exist").path,
            "archive/ish-done--completed.md"
        );
        assert_eq!(
            store
                .get("ish-nope")
                .expect("scrapped ish should exist")
                .path,
            "archive/ish-nope--scrapped.md"
        );
    }

    #[test]
    fn create_writes_new_ish_to_disk_and_store() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("root dir should exist");
        write_ish(
            &root.join("ish-parent--parent.md"),
            "ish-parent",
            "Parent",
            "todo",
            "feature",
            Some("normal"),
            "Parent body.",
        );
        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        let created = store
            .create(CreateIsh {
                title: "Create store record".to_string(),
                status: None,
                ish_type: Some("bug".to_string()),
                priority: Some("high".to_string()),
                body: "Created body.".to_string(),
                tags: vec!["Backend".to_string(), "backend".to_string()],
                parent: Some("parent".to_string()),
                blocking: vec!["dep1".to_string(), "dep1".to_string()],
                blocked_by: vec!["dep2".to_string()],
                id_prefix: None,
            })
            .expect("create should succeed");

        let file_path = root.join(&created.path);
        let file_contents = fs::read_to_string(&file_path).expect("created file should exist");

        assert!(created.id.starts_with("ish-"));
        assert_eq!(created.slug, "create-store-record");
        assert_eq!(created.status, "todo");
        assert_eq!(created.ish_type, "bug");
        assert_eq!(created.priority.as_deref(), Some("high"));
        assert_eq!(created.tags, vec!["backend"]);
        assert_eq!(created.parent.as_deref(), Some("ish-parent"));
        assert_eq!(created.blocking, vec!["ish-dep1"]);
        assert_eq!(created.blocked_by, vec!["ish-dep2"]);
        assert!(file_contents.contains("title: Create store record"));
        assert!(file_contents.contains("Created body."));
        assert_eq!(
            store.get(&created.id).expect("created ish should exist"),
            &created
        );
    }

    #[test]
    fn update_applies_field_changes_and_renames_file() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        let original_path = root.join("ish-abcd--old-title.md");

        fs::create_dir_all(&root).expect("root dir should exist");
        write_ish(
            &root.join("ish-parent--parent.md"),
            "ish-parent",
            "Parent",
            "todo",
            "epic",
            Some("normal"),
            "Parent body.",
        );
        write_ish(
            &original_path,
            "ish-abcd",
            "Old title",
            "todo",
            "task",
            Some("normal"),
            "alpha target omega",
        );

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");
        let etag = store.get("ish-abcd").expect("ish should exist").etag();

        let updated = store
            .update(
                "abcd",
                UpdateIsh {
                    status: Some("in-progress".to_string()),
                    ish_type: Some("feature".to_string()),
                    priority: Some(Some("critical".to_string())),
                    title: Some("New title".to_string()),
                    body: None,
                    body_replace: Some(("target".to_string(), "updated".to_string())),
                    body_append: Some("appended text".to_string()),
                    add_tags: vec!["cli".to_string()],
                    remove_tags: Vec::new(),
                    parent: Some(Some("parent".to_string())),
                    add_blocking: vec!["child".to_string()],
                    remove_blocking: Vec::new(),
                    add_blocked_by: vec!["dep".to_string()],
                    remove_blocked_by: Vec::new(),
                    if_match: Some(etag),
                },
            )
            .expect("update should succeed");

        let renamed_path = root.join("ish-abcd--new-title.md");
        let file_contents = fs::read_to_string(&renamed_path).expect("renamed file should exist");

        assert!(!original_path.exists());
        assert!(renamed_path.exists());
        assert_eq!(updated.status, "in-progress");
        assert_eq!(updated.ish_type, "feature");
        assert_eq!(updated.priority.as_deref(), Some("critical"));
        assert_eq!(updated.slug, "new-title");
        assert_eq!(updated.path, "ish-abcd--new-title.md");
        assert_eq!(updated.tags, vec!["cli"]);
        assert_eq!(updated.parent.as_deref(), Some("ish-parent"));
        assert_eq!(updated.blocking, vec!["ish-child"]);
        assert_eq!(updated.blocked_by, vec!["ish-dep"]);
        assert_eq!(updated.body, "alpha updated omega\n\nappended text");
        assert!(file_contents.contains("alpha updated omega"));
        assert!(file_contents.contains("appended text"));
    }

    #[test]
    fn update_rejects_etag_mismatch() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("root dir should exist");
        write_ish(
            &root.join("ish-abcd--etag.md"),
            "ish-abcd",
            "ETag",
            "todo",
            "task",
            Some("normal"),
            "Body.",
        );

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        let error = store
            .update(
                "ish-abcd",
                UpdateIsh {
                    title: Some("Other".to_string()),
                    if_match: Some("deadbeefdeadbeef".to_string()),
                    ..UpdateIsh::default()
                },
            )
            .expect_err("update should fail on mismatched etag");

        assert!(matches!(error, StoreError::ETagMismatch { .. }));
    }

    #[test]
    fn valid_parent_types_match_expected_hierarchy() {
        assert_eq!(Store::valid_parent_types("milestone"), &[] as &[&str]);
        assert_eq!(Store::valid_parent_types("epic"), &["milestone"]);
        assert_eq!(Store::valid_parent_types("feature"), &["milestone", "epic"]);
        assert_eq!(
            Store::valid_parent_types("task"),
            &["milestone", "epic", "feature"]
        );
        assert_eq!(
            Store::valid_parent_types("bug"),
            &["milestone", "epic", "feature"]
        );
    }

    #[test]
    fn create_rejects_invalid_parent_hierarchy() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");

        fs::create_dir_all(&root).expect("root dir should exist");
        write_ish(
            &root.join("ish-task-parent--task-parent.md"),
            "ish-task-parent",
            "Task parent",
            "todo",
            "task",
            Some("normal"),
            "Task parent body.",
        );

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        let error = store
            .create(CreateIsh {
                title: "Feature child".to_string(),
                ish_type: Some("feature".to_string()),
                parent: Some("task-parent".to_string()),
                ..CreateIsh::default()
            })
            .expect_err("create should reject invalid parent hierarchy");

        assert!(matches!(
            error,
            StoreError::InvalidParentType {
                child_type,
                parent_id,
                parent_type,
                allowed_parent_types,
            } if child_type == "feature"
                && parent_id == "ish-task-parent"
                && parent_type == "task"
                && allowed_parent_types == vec!["milestone", "epic"]
        ));
    }

    #[test]
    fn create_rejects_parent_for_milestone() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");

        fs::create_dir_all(&root).expect("root dir should exist");
        write_ish(
            &root.join("ish-parent--parent.md"),
            "ish-parent",
            "Parent",
            "todo",
            "milestone",
            Some("normal"),
            "Parent body.",
        );

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        let error = store
            .create(CreateIsh {
                title: "Milestone child".to_string(),
                ish_type: Some("milestone".to_string()),
                parent: Some("parent".to_string()),
                ..CreateIsh::default()
            })
            .expect_err("milestone should reject parent assignments");

        assert!(
            matches!(error, StoreError::ParentNotAllowed(ref ish_type) if ish_type == "milestone")
        );
    }

    #[test]
    fn update_rejects_invalid_parent_hierarchy_after_type_change() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");

        fs::create_dir_all(&root).expect("root dir should exist");
        write_ish(
            &root.join("ish-parent--parent.md"),
            "ish-parent",
            "Parent",
            "todo",
            "feature",
            Some("normal"),
            "Parent body.",
        );
        write_ish(
            &root.join("ish-child--child.md"),
            "ish-child",
            "Child",
            "todo",
            "task",
            Some("normal"),
            "Child body.",
        );

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        let error = store
            .update(
                "child",
                UpdateIsh {
                    ish_type: Some("epic".to_string()),
                    parent: Some(Some("parent".to_string())),
                    ..UpdateIsh::default()
                },
            )
            .expect_err("update should reject invalid parent hierarchy");

        assert!(matches!(
            error,
            StoreError::InvalidParentType {
                child_type,
                parent_id,
                parent_type,
                allowed_parent_types,
            } if child_type == "epic"
                && parent_id == "ish-parent"
                && parent_type == "feature"
                && allowed_parent_types == vec!["milestone"]
        ));
    }

    #[test]
    fn delete_removes_file_and_cleans_incoming_references() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("root dir should exist");
        fs::write(
            root.join("ish-target--target.md"),
            "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTarget body.\n",
        )
        .expect("target file should be written");
        fs::write(
            root.join("ish-ref--ref.md"),
            "---\n# ish-ref\ntitle: Ref\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-target\nblocking:\n  - ish-target\nblocked_by:\n  - ish-target\n---\n\nRef body.\n",
        )
        .expect("ref file should be written");

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        let removed = store.delete("target").expect("delete should succeed");
        let ref_ish = store.get("ish-ref").expect("ref should remain");
        let ref_contents =
            fs::read_to_string(root.join("ish-ref--ref.md")).expect("ref file should still exist");

        assert_eq!(removed.id, "ish-target");
        assert!(!root.join("ish-target--target.md").exists());
        assert!(ref_ish.parent.is_none());
        assert!(ref_ish.blocking.is_empty());
        assert!(ref_ish.blocked_by.is_empty());
        assert!(!ref_contents.contains("parent: ish-target"));
        assert!(!ref_contents.contains("- ish-target"));
    }

    #[test]
    fn detect_cycle_finds_cycles_per_link_type() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("root dir should exist");
        fs::write(
            root.join("ish-a--a.md"),
            "---\n# ish-a\ntitle: A\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-b\n---\n\nA body.\n",
        )
        .expect("a file should be written");
        fs::write(
            root.join("ish-b--b.md"),
            "---\n# ish-b\ntitle: B\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-c\n---\n\nB body.\n",
        )
        .expect("b file should be written");
        fs::write(
            root.join("ish-c--c.md"),
            "---\n# ish-c\ntitle: C\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nC body.\n",
        )
        .expect("c file should be written");

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        assert!(store.detect_cycle("ish-c", LinkType::Blocking, "ish-a"));
        assert!(!store.detect_cycle("ish-c", LinkType::BlockedBy, "ish-a"));
    }

    #[test]
    fn find_incoming_links_returns_all_matching_link_types() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("root dir should exist");
        fs::write(
            root.join("ish-target--target.md"),
            "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTarget body.\n",
        )
        .expect("target file should be written");
        fs::write(
            root.join("ish-parented--parented.md"),
            "---\n# ish-parented\ntitle: Parented\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-target\n---\n\nParented body.\n",
        )
        .expect("parented file should be written");
        fs::write(
            root.join("ish-blocker--blocker.md"),
            "---\n# ish-blocker\ntitle: Blocker\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-target\n---\n\nBlocker body.\n",
        )
        .expect("blocker file should be written");
        fs::write(
            root.join("ish-blocked--blocked.md"),
            "---\n# ish-blocked\ntitle: Blocked\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocked_by:\n  - ish-target\n---\n\nBlocked body.\n",
        )
        .expect("blocked file should be written");

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        assert_eq!(
            store.find_incoming_links("target"),
            vec![
                LinkRef {
                    source_id: "ish-blocked".to_string(),
                    link_type: LinkType::BlockedBy,
                    target_id: "ish-target".to_string(),
                },
                LinkRef {
                    source_id: "ish-blocker".to_string(),
                    link_type: LinkType::Blocking,
                    target_id: "ish-target".to_string(),
                },
                LinkRef {
                    source_id: "ish-parented".to_string(),
                    link_type: LinkType::Parent,
                    target_id: "ish-target".to_string(),
                },
            ]
        );
    }

    #[test]
    fn check_all_links_reports_broken_self_and_cycle_links() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("root dir should exist");
        fs::write(
            root.join("ish-a--a.md"),
            "---\n# ish-a\ntitle: A\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-b\n---\n\nA body.\n",
        )
        .expect("a file should be written");
        fs::write(
            root.join("ish-b--b.md"),
            "---\n# ish-b\ntitle: B\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-c\n---\n\nB body.\n",
        )
        .expect("b file should be written");
        fs::write(
            root.join("ish-c--c.md"),
            "---\n# ish-c\ntitle: C\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-a\n---\n\nC body.\n",
        )
        .expect("c file should be written");
        fs::write(
            root.join("ish-bad--bad.md"),
            "---\n# ish-bad\ntitle: Bad\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-missing\nblocking:\n  - ish-bad\nblocked_by:\n  - ish-missing-two\n---\n\nBad body.\n",
        )
        .expect("bad file should be written");

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        assert_eq!(
            store.check_all_links(),
            LinkCheckResult {
                broken_links: vec![
                    LinkRef {
                        source_id: "ish-bad".to_string(),
                        link_type: LinkType::Parent,
                        target_id: "ish-missing".to_string(),
                    },
                    LinkRef {
                        source_id: "ish-bad".to_string(),
                        link_type: LinkType::BlockedBy,
                        target_id: "ish-missing-two".to_string(),
                    },
                ],
                self_links: vec![LinkRef {
                    source_id: "ish-bad".to_string(),
                    link_type: LinkType::Blocking,
                    target_id: "ish-bad".to_string(),
                }],
                cycles: vec![LinkCycle {
                    link_type: LinkType::Blocking,
                    path: vec![
                        "ish-a".to_string(),
                        "ish-b".to_string(),
                        "ish-c".to_string(),
                        "ish-a".to_string(),
                    ],
                }],
            }
        );
    }

    #[test]
    fn fix_broken_links_removes_invalid_references_and_saves_files() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("root dir should exist");
        fs::write(
            root.join("ish-valid--valid.md"),
            "---\n# ish-valid\ntitle: Valid\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nValid body.\n",
        )
        .expect("valid file should be written");
        fs::write(
            root.join("ish-bad--bad.md"),
            "---\n# ish-bad\ntitle: Bad\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-bad\nblocking:\n  - ish-valid\n  - ish-bad\n  - ish-missing\nblocked_by:\n  - ish-bad\n  - ish-missing-two\n---\n\nBad body.\n",
        )
        .expect("bad file should be written");

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        let fixed = store
            .fix_broken_links()
            .expect("fixing broken links should succeed");
        let ish = store.get("ish-bad").expect("bad ish should exist");
        let contents =
            fs::read_to_string(root.join("ish-bad--bad.md")).expect("bad file should still exist");

        assert_eq!(fixed, 5);
        assert!(ish.parent.is_none());
        assert_eq!(ish.blocking, vec!["ish-valid"]);
        assert!(ish.blocked_by.is_empty());
        assert!(!contents.contains("parent: ish-bad"));
        assert!(contents.contains("- ish-valid"));
        assert!(!contents.contains("- ish-missing"));
        assert!(!contents.contains("- ish-bad"));
        assert!(!contents.contains("- ish-missing-two"));
    }

    #[test]
    fn blocker_queries_include_direct_blockers_from_both_link_directions() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("root dir should exist");
        fs::write(
            root.join("ish-target--target.md"),
            "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocked_by:\n  - ish-listed\n---\n\nTarget body.\n",
        )
        .expect("target file should be written");
        fs::write(
            root.join("ish-listed--listed.md"),
            "---\n# ish-listed\ntitle: Listed\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nListed body.\n",
        )
        .expect("listed blocker file should be written");
        fs::write(
            root.join("ish-incoming--incoming.md"),
            "---\n# ish-incoming\ntitle: Incoming\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-target\n---\n\nIncoming body.\n",
        )
        .expect("incoming blocker file should be written");
        fs::write(
            root.join("ish-resolved--resolved.md"),
            "---\n# ish-resolved\ntitle: Resolved\nstatus: completed\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-target\n---\n\nResolved body.\n",
        )
        .expect("resolved blocker file should be written");

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        assert_eq!(
            store.find_active_blockers("target"),
            vec!["ish-incoming".to_string(), "ish-listed".to_string()]
        );
        assert!(store.is_explicitly_blocked("target"));
    }

    #[test]
    fn blocker_queries_include_ancestor_blockers() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("root dir should exist");
        fs::write(
            root.join("ish-parent--parent.md"),
            "---\n# ish-parent\ntitle: Parent\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocked_by:\n  - ish-parent-blocker\n---\n\nParent body.\n",
        )
        .expect("parent file should be written");
        fs::write(
            root.join("ish-child--child.md"),
            "---\n# ish-child\ntitle: Child\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-parent\n---\n\nChild body.\n",
        )
        .expect("child file should be written");
        fs::write(
            root.join("ish-parent-blocker--parent-blocker.md"),
            "---\n# ish-parent-blocker\ntitle: Parent Blocker\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nParent blocker body.\n",
        )
        .expect("parent blocker file should be written");

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        assert_eq!(
            store.find_all_blockers("child"),
            vec!["ish-parent-blocker".to_string()]
        );
        assert!(store.is_blocked("child"));
        assert!(!store.is_explicitly_blocked("child"));
        assert!(store.is_implicitly_blocked("child"));
    }

    #[test]
    fn implicit_status_returns_first_terminal_ancestor() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("root dir should exist");
        fs::write(
            root.join("ish-grandparent--grandparent.md"),
            "---\n# ish-grandparent\ntitle: Grandparent\nstatus: completed\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nGrandparent body.\n",
        )
        .expect("grandparent file should be written");
        fs::write(
            root.join("ish-parent--parent.md"),
            "---\n# ish-parent\ntitle: Parent\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-grandparent\n---\n\nParent body.\n",
        )
        .expect("parent file should be written");
        fs::write(
            root.join("ish-child--child.md"),
            "---\n# ish-child\ntitle: Child\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-parent\n---\n\nChild body.\n",
        )
        .expect("child file should be written");

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        assert_eq!(
            store.implicit_status("child"),
            Some(("completed".to_string(), "ish-grandparent".to_string()))
        );
    }

    #[test]
    fn parent_chain_cycle_does_not_loop_when_finding_inherited_state() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("root dir should exist");
        fs::write(
            root.join("ish-a--a.md"),
            "---\n# ish-a\ntitle: A\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-b\nblocked_by:\n  - ish-blocker\n---\n\nA body.\n",
        )
        .expect("a file should be written");
        fs::write(
            root.join("ish-b--b.md"),
            "---\n# ish-b\ntitle: B\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-a\n---\n\nB body.\n",
        )
        .expect("b file should be written");
        fs::write(
            root.join("ish-blocker--blocker.md"),
            "---\n# ish-blocker\ntitle: Blocker\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nBlocker body.\n",
        )
        .expect("blocker file should be written");

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        assert_eq!(
            store.find_all_blockers("b"),
            vec!["ish-blocker".to_string()]
        );
        assert!(store.is_implicitly_blocked("b"));
        assert_eq!(store.implicit_status("b"), None);
    }

    fn write_ish(
        path: &Path,
        id: &str,
        title: &str,
        status: &str,
        ish_type: &str,
        priority: Option<&str>,
        body: &str,
    ) {
        let priority_line = priority
            .map(|priority| format!("priority: {priority}\n"))
            .unwrap_or_default();
        let contents = format!(
            "---\n# {id}\ntitle: {title}\nstatus: {status}\ntype: {ish_type}\n{priority_line}created_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\n{body}\n"
        );
        fs::write(path, contents).expect("ish file should be written");
    }
}
