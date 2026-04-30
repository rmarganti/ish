use crate::config::Config;
use crate::model::ish::{
    BodyError, Ish, append_with_separator, build_filename, new_id, replace_once, slugify,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveWarningKind {
    ActiveChildWithArchivedParent,
    ActiveIshReferencesArchivedIsh,
    ArchivedIshReferencesActiveIsh,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct ArchiveWarning {
    pub kind: ArchiveWarningKind,
    pub source_id: String,
    pub link_type: LinkType,
    pub target_id: String,
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
        Ok(ish.is_archived())
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
            .filter(|ish| self.config.is_archive_status(&ish.status) && !ish.is_archived())
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
                && !candidate.is_archived()
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

    pub fn find_archive_warnings(&self) -> Vec<ArchiveWarning> {
        let mut warnings = HashSet::new();

        for ish in self.ishes.values() {
            let ish_archived = ish.is_archived();

            for link in collect_links(ish) {
                let Some(target) = self.ishes.get(&link.target_id) else {
                    continue;
                };

                match (ish_archived, target.is_archived(), link.link_type) {
                    (false, true, LinkType::Parent) => {
                        warnings.insert(ArchiveWarning {
                            kind: ArchiveWarningKind::ActiveChildWithArchivedParent,
                            source_id: link.source_id,
                            link_type: link.link_type,
                            target_id: link.target_id,
                        });
                    }
                    (false, true, _) => {
                        warnings.insert(ArchiveWarning {
                            kind: ArchiveWarningKind::ActiveIshReferencesArchivedIsh,
                            source_id: link.source_id,
                            link_type: link.link_type,
                            target_id: link.target_id,
                        });
                    }
                    (true, false, _) => {
                        warnings.insert(ArchiveWarning {
                            kind: ArchiveWarningKind::ArchivedIshReferencesActiveIsh,
                            source_id: link.source_id,
                            link_type: link.link_type,
                            target_id: link.target_id,
                        });
                    }
                    _ => {}
                }
            }
        }

        let mut warnings = warnings.into_iter().collect::<Vec<_>>();
        warnings.sort_by(|left, right| {
            archive_warning_kind_rank(left.kind)
                .cmp(&archive_warning_kind_rank(right.kind))
                .then_with(|| left.source_id.cmp(&right.source_id))
                .then_with(|| link_type_rank(left.link_type).cmp(&link_type_rank(right.link_type)))
                .then_with(|| left.target_id.cmp(&right.target_id))
        });
        warnings
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
            .is_some_and(|ish| !ish.is_archived() && !self.config.is_archive_status(&ish.status))
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

fn archive_warning_kind_rank(kind: ArchiveWarningKind) -> usize {
    match kind {
        ArchiveWarningKind::ActiveChildWithArchivedParent => 0,
        ArchiveWarningKind::ActiveIshReferencesArchivedIsh => 1,
        ArchiveWarningKind::ArchivedIshReferencesActiveIsh => 2,
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
mod tests;
