use crate::app::{AppContext, AppError, json_output_error, store_app_error};
use crate::cli::DeleteArgs;
use crate::core::store::{LinkRef, Store, StoreError};
use crate::model::ish::Ish;
use crate::output::{ErrorCode, muted, output_success, render_id, success, warning};
use serde::Serialize;
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone)]
pub(crate) struct DeleteTarget {
    pub(crate) ish: Ish,
    pub(crate) incoming_links: Vec<LinkRef>,
}

#[derive(Debug, Serialize)]
struct DeleteJson {
    deleted: Vec<crate::model::ish::IshJson>,
    count: usize,
    cleaned_links: usize,
}

pub(crate) fn delete_command(args: DeleteArgs, json: bool) -> Result<Option<String>, AppError> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    delete_command_with_io(args, json, &mut stdin, &mut stdout)
}

pub(crate) fn delete_command_with_io<R: BufRead, W: Write>(
    args: DeleteArgs,
    json: bool,
    input: &mut R,
    output: &mut W,
) -> Result<Option<String>, AppError> {
    let mut store = AppContext::load()?.store;
    let targets = resolve_delete_targets(&store, &args.ids)?;
    let cleaned_links = targets
        .iter()
        .map(|target| target.incoming_links.len())
        .sum::<usize>();

    if !args.force && !json && !confirm_delete(&targets, input, output)? {
        return Ok(Some(warning("Delete cancelled")));
    }

    let mut deleted = Vec::with_capacity(targets.len());
    for target in &targets {
        deleted.push(store.delete(&target.ish.id).map_err(store_app_error)?);
    }

    if json {
        return Ok(Some(
            output_success(DeleteJson {
                deleted: deleted.iter().map(|ish| ish.to_json(&ish.etag())).collect(),
                count: deleted.len(),
                cleaned_links,
            })
            .map_err(json_output_error)?,
        ));
    }

    Ok(Some(render_delete_success(&deleted, cleaned_links)))
}

fn resolve_delete_targets(store: &Store, ids: &[String]) -> Result<Vec<DeleteTarget>, AppError> {
    let mut ordered_ids = Vec::new();
    let mut seen = HashSet::new();

    for id in ids {
        let normalized = store.normalize_id(id);
        if seen.insert(normalized.clone()) {
            ordered_ids.push(normalized);
        }
    }

    let target_ids = ordered_ids.iter().cloned().collect::<HashSet<_>>();
    ordered_ids
        .into_iter()
        .map(|id| {
            let ish = store
                .get(&id)
                .cloned()
                .ok_or_else(|| store_app_error(StoreError::NotFound(id.clone())))?;
            let incoming_links = store
                .find_incoming_links(&id)
                .into_iter()
                .filter(|link| !target_ids.contains(&link.source_id))
                .collect();

            Ok(DeleteTarget {
                ish,
                incoming_links,
            })
        })
        .collect()
}

pub(crate) fn confirm_delete<R: BufRead, W: Write>(
    targets: &[DeleteTarget],
    input: &mut R,
    output: &mut W,
) -> Result<bool, AppError> {
    let total_incoming = targets
        .iter()
        .map(|target| target.incoming_links.len())
        .sum::<usize>();
    let issue_label = if targets.len() == 1 { "ish" } else { "ishes" };

    writeln!(output, "Delete {} {issue_label}?", targets.len()).map_err(prompt_io_error)?;
    for target in targets {
        writeln!(
            output,
            "- {} | title: {} | path: {} | incoming links: {}",
            target.ish.id,
            target.ish.title,
            target.ish.path,
            target.incoming_links.len()
        )
        .map_err(prompt_io_error)?;
    }

    if total_incoming > 0 {
        writeln!(
            output,
            "Warning: deleting these ishes will remove {total_incoming} incoming link(s) from remaining ishs."
        )
        .map_err(prompt_io_error)?;
    }

    write!(output, "Proceed? [y/N]: ").map_err(prompt_io_error)?;
    output.flush().map_err(prompt_io_error)?;

    let mut response = String::new();
    input.read_line(&mut response).map_err(prompt_io_error)?;
    let response = response.trim().to_ascii_lowercase();

    Ok(matches!(response.as_str(), "y" | "yes"))
}

fn render_delete_success(deleted: &[Ish], cleaned_links: usize) -> String {
    if deleted.len() == 1 {
        let deleted = &deleted[0];
        let suffix = if cleaned_links == 0 {
            String::new()
        } else {
            format!(" and cleaned {cleaned_links} incoming link(s)")
        };
        return success(&format!(
            "Deleted {} {}{suffix}",
            render_id(&deleted.id),
            muted(&deleted.path)
        ));
    }

    let suffix = if cleaned_links == 0 {
        String::new()
    } else {
        format!(" and cleaned {cleaned_links} incoming link(s)")
    };
    success(&format!("Deleted {} ishes{suffix}", deleted.len()))
}

fn prompt_io_error(error: io::Error) -> AppError {
    AppError::new(
        ErrorCode::FileError,
        format!("failed to read delete confirmation: {error}"),
    )
}

#[cfg(test)]
mod tests {
    use super::{DeleteTarget, confirm_delete, delete_command_with_io};
    use crate::cli::DeleteArgs;
    use crate::config::Config;
    use crate::core::store::{LinkRef, LinkType};
    use crate::model::ish::Ish;
    use crate::test_support::{TestDir, WorkingDirGuard};
    use chrono::Utc;
    use serde_json::Value;
    use std::fs;
    use std::io::Cursor;

    fn sample_delete_target(incoming_links: usize) -> DeleteTarget {
        DeleteTarget {
            ish: Ish {
                id: "ish-target".to_string(),
                slug: "target".to_string(),
                path: "ish-target--target.md".to_string(),
                title: "Target".to_string(),
                status: "todo".to_string(),
                ish_type: "task".to_string(),
                priority: Some("normal".to_string()),
                tags: Vec::new(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                order: None,
                body: "Target body.".to_string(),
                parent: None,
                blocking: Vec::new(),
                blocked_by: Vec::new(),
            },
            incoming_links: (0..incoming_links)
                .map(|index| LinkRef {
                    source_id: format!("ish-ref-{index}"),
                    link_type: LinkType::Blocking,
                    target_id: "ish-target".to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn confirm_delete_prints_title_path_and_incoming_link_count() {
        let mut input = Cursor::new(b"yes\n".to_vec());
        let mut output = Vec::new();

        let confirmed = confirm_delete(&[sample_delete_target(2)], &mut input, &mut output)
            .expect("confirmation prompt should succeed");
        let rendered = String::from_utf8(output).expect("prompt should be utf8");

        assert!(confirmed);
        assert!(rendered.contains("Delete 1 ish?"));
        assert!(rendered.contains("title: Target"));
        assert!(rendered.contains("path: ish-target--target.md"));
        assert!(rendered.contains("incoming links: 2"));
        assert!(rendered.contains("remove 2 incoming link(s)"));
        assert!(rendered.contains("Proceed? [y/N]:"));
    }

    #[test]
    fn delete_command_force_removes_file_and_cleans_references() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-target--target.md"),
            "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTarget body.\n",
        )
        .expect("target file should exist");
        fs::write(
            store_root.join("ish-ref--ref.md"),
            "---\n# ish-ref\ntitle: Ref\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-target\nblocking:\n  - ish-target\nblocked_by:\n  - ish-target\n---\n\nRef body.\n",
        )
        .expect("ref file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = delete_command_with_io(
            DeleteArgs {
                ids: vec!["target".to_string()],
                force: true,
            },
            false,
            &mut Cursor::new(Vec::new()),
            &mut Vec::new(),
        )
        .expect("delete command should succeed")
        .expect("delete command should print output");

        assert!(output.contains("Deleted"));
        assert!(output.contains("cleaned 3 incoming link(s)"));
        assert!(!store_root.join("ish-target--target.md").exists());

        let ref_contents =
            fs::read_to_string(store_root.join("ish-ref--ref.md")).expect("ref file should exist");
        assert!(!ref_contents.contains("parent: ish-target"));
        assert!(!ref_contents.contains("- ish-target"));
    }

    #[test]
    fn delete_command_returns_cancelled_message_without_deleting() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-target--target.md"),
            "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTarget body.\n",
        )
        .expect("target file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());
        let mut prompt_output = Vec::new();

        let output = delete_command_with_io(
            DeleteArgs {
                ids: vec!["target".to_string()],
                force: false,
            },
            false,
            &mut Cursor::new(b"n\n".to_vec()),
            &mut prompt_output,
        )
        .expect("delete command should succeed")
        .expect("delete command should print output");

        assert!(output.contains("Delete cancelled"));
        assert!(store_root.join("ish-target--target.md").exists());
        let prompt_output = String::from_utf8(prompt_output).expect("prompt should be utf8");
        assert!(prompt_output.contains("title: Target"));
    }

    #[test]
    fn delete_command_json_implies_force_and_returns_deleted_items() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-target--target.md"),
            "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTarget body.\n",
        )
        .expect("target file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());
        let mut prompt_output = Vec::new();

        let output = delete_command_with_io(
            DeleteArgs {
                ids: vec!["target".to_string()],
                force: false,
            },
            true,
            &mut Cursor::new(Vec::new()),
            &mut prompt_output,
        )
        .expect("delete command should succeed")
        .expect("delete command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["data"]["count"], Value::from(1));
        assert_eq!(parsed["data"]["cleaned_links"], Value::from(0));
        assert_eq!(parsed["data"]["deleted"][0]["id"], "ish-target");
        assert!(prompt_output.is_empty());
        assert!(!store_root.join("ish-target--target.md").exists());
    }
}
