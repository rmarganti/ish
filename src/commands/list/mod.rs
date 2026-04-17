use crate::app::{AppContext, AppError, json_output_error};
use crate::cli::{ListArgs, ListSortArg};
use crate::core::SortMode;
use crate::output::{output_success_multiple, warning};

mod filters;
mod render;

use filters::{filter_ishoos, sort_ishoo_refs, validate_list_args};
use render::{list_json_value, render_tree_output};

pub(crate) fn list_command(args: ListArgs, json: bool) -> Result<Option<String>, AppError> {
    let context = AppContext::load()?;
    let config = context.config;
    let store = context.store;
    validate_list_args(&args, &config)?;

    let all_ishoos = store.all().into_iter().cloned().collect::<Vec<_>>();
    let filtered = filter_ishoos(&all_ishoos, &store, &config, &args);
    let sort_mode = args
        .sort
        .map(ListSortArg::into_sort_mode)
        .unwrap_or(SortMode::Default);
    let sorted = sort_ishoo_refs(&filtered, sort_mode, &config);

    if json {
        let ishoos = sorted
            .into_iter()
            .map(|ishoo| list_json_value(ishoo, args.full))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(Some(
            output_success_multiple(ishoos).map_err(json_output_error)?,
        ));
    }

    if args.quiet {
        let output = sorted
            .iter()
            .map(|ishoo| ishoo.id.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        return Ok((!output.is_empty()).then_some(output));
    }

    if sorted.is_empty() {
        return Ok(Some(warning("No ishoos found")));
    }

    Ok(Some(render_tree_output(
        &sorted,
        &all_ishoos,
        &store,
        &config,
        sort_mode,
    )))
}
