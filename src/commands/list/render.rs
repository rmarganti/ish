use crate::app::AppError;
use crate::config::Config;
use crate::core::SortMode;
use crate::core::store::Store;
use crate::model::ishoo::Ishoo;
use crate::output::{ErrorCode, TreeNode, build_tree, detect_terminal_width, render_tree};
use serde_json::{Value, to_value};
use std::collections::HashMap;

use super::filters::sort_ishoo_refs;

pub(super) fn render_tree_output(
    sorted: &[&Ishoo],
    all_ishoos: &[Ishoo],
    store: &Store,
    config: &Config,
    sort_mode: SortMode,
) -> String {
    let all_refs = all_ishoos.iter().collect::<Vec<_>>();
    let implicit_statuses = sorted
        .iter()
        .filter_map(|ishoo| {
            store
                .implicit_status(&ishoo.id)
                .map(|(status, _)| (ishoo.id.clone(), status))
        })
        .collect::<HashMap<_, _>>();
    let tree = build_tree(
        sorted,
        &all_refs,
        |items| sort_ishoo_refs(items, sort_mode, config),
        &implicit_statuses,
    );

    render_tree(
        &tree,
        config,
        max_tree_id_width(&tree),
        tree_has_tags(&tree),
        detect_terminal_width(),
    )
}

pub(super) fn list_json_value(ishoo: &Ishoo, full: bool) -> Result<Value, AppError> {
    let mut value = to_value(ishoo.to_json(&ishoo.etag())).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to serialize list output: {error}"),
        )
    })?;

    if !full && let Some(object) = value.as_object_mut() {
        object.remove("body");
    }

    Ok(value)
}

fn tree_has_tags(tree: &[TreeNode<'_>]) -> bool {
    tree.iter()
        .any(|node| !node.ishoo.tags.is_empty() || tree_has_tags(&node.children))
}

fn max_tree_id_width(tree: &[TreeNode<'_>]) -> usize {
    tree.iter()
        .map(|node| node.ishoo.id.len().max(max_tree_id_width(&node.children)))
        .max()
        .unwrap_or(0)
}
