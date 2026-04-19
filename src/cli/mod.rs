use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::core::SortMode;

/// A terminal-based issue tracker.
#[derive(Parser)]
#[command(name = "ish", version, about, arg_required_else_help = true)]
pub struct Cli {
    /// Output structured JSON.
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new ish project in the current directory.
    Init,
    /// Create a new ish markdown file.
    Create(CreateArgs),
    /// List ishes, optionally filtered and sorted.
    #[command(visible_alias = "ls")]
    List(ListArgs),
    /// Update an existing ish.
    #[command(visible_alias = "u")]
    Update(UpdateArgs),
    /// Show one or more ishes in detail.
    Show(ShowArgs),
    /// Delete one or more ishes.
    #[command(visible_alias = "rm")]
    Delete(DeleteArgs),
    /// Move completed and scrapped ishes to the archive directory.
    Archive,
    /// Validate configuration and link integrity.
    Check(CheckArgs),
    /// Print AI-agent guidance for the current ish project.
    Prime,
    /// Generate a roadmap from milestone and epic hierarchy.
    Roadmap(RoadmapArgs),
    /// Print the current ish version.
    Version,
}

#[derive(Args)]
pub struct RoadmapArgs {
    /// Include completed and scrapped items.
    #[arg(long)]
    pub include_done: bool,
    /// Filter milestones by status.
    #[arg(long = "status")]
    pub status: Vec<String>,
    /// Exclude milestones by status.
    #[arg(long = "no-status")]
    pub no_status: Vec<String>,
    /// Render plain IDs instead of markdown links.
    #[arg(long)]
    pub no_links: bool,
    /// Override the link prefix used in markdown links.
    #[arg(long)]
    pub link_prefix: Option<String>,
}

#[derive(Args)]
pub struct CheckArgs {
    /// Fix broken links and self-references.
    #[arg(long)]
    pub fix: bool,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Title for the new ish.
    pub title: Option<String>,
    /// Override the initial status.
    #[arg(short = 's', long = "status")]
    pub status: Option<String>,
    /// Override the ish type.
    #[arg(short = 't', long = "type")]
    pub ish_type: Option<String>,
    /// Override the priority.
    #[arg(short = 'p', long = "priority")]
    pub priority: Option<String>,
    /// Inline body text; use `-` to read from stdin.
    #[arg(short = 'd', long = "body", conflicts_with = "body_file")]
    pub body: Option<String>,
    /// Read body text from a file.
    #[arg(long = "body-file", conflicts_with = "body")]
    pub body_file: Option<String>,
    /// Add a tag. May be repeated.
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    /// Set the parent ish ID.
    #[arg(long = "parent")]
    pub parent: Option<String>,
    /// Add a blocking relationship. May be repeated.
    #[arg(long = "blocking")]
    pub blocking: Vec<String>,
    /// Add a blocked-by relationship. May be repeated.
    #[arg(long = "blocked-by")]
    pub blocked_by: Vec<String>,
    /// Override the ID prefix for the created ish.
    #[arg(long = "prefix")]
    pub prefix: Option<String>,
}

#[derive(Args)]
pub struct ListArgs {
    /// Filter by status. May be repeated.
    #[arg(short = 's', long = "status")]
    pub status: Vec<String>,
    /// Exclude statuses. May be repeated.
    #[arg(long = "no-status")]
    pub no_status: Vec<String>,
    /// Filter by type. May be repeated.
    #[arg(short = 't', long = "type")]
    pub ish_type: Vec<String>,
    /// Exclude types. May be repeated.
    #[arg(long = "no-type")]
    pub no_type: Vec<String>,
    /// Filter by priority. May be repeated.
    #[arg(short = 'p', long = "priority")]
    pub priority: Vec<String>,
    /// Exclude priorities. May be repeated.
    #[arg(long = "no-priority")]
    pub no_priority: Vec<String>,
    /// Match any tag. May be repeated.
    #[arg(long = "tag")]
    pub tag: Vec<String>,
    /// Exclude any matching tag. May be repeated.
    #[arg(long = "no-tag")]
    pub no_tag: Vec<String>,
    /// Only include ishes with a parent.
    #[arg(long, conflicts_with_all = ["no_parent", "parent"])]
    pub has_parent: bool,
    /// Only include ishes without a parent.
    #[arg(long, conflicts_with_all = ["has_parent", "parent"])]
    pub no_parent: bool,
    /// Only include children of the specified parent.
    #[arg(long = "parent", conflicts_with_all = ["has_parent", "no_parent"])]
    pub parent: Option<String>,
    /// Only include ishes that block other ishs.
    #[arg(long, conflicts_with = "no_blocking")]
    pub has_blocking: bool,
    /// Only include ishes with no blocking links.
    #[arg(long, conflicts_with = "has_blocking")]
    pub no_blocking: bool,
    /// Only include blocked ishes.
    #[arg(long)]
    pub is_blocked: bool,
    /// Only include ready ishes.
    #[arg(long)]
    pub ready: bool,
    /// Case-insensitive substring search.
    #[arg(short = 'S', long = "search")]
    pub search: Option<String>,
    /// Sort mode.
    #[arg(long = "sort")]
    pub sort: Option<ListSortArg>,
    /// Print only IDs.
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
    /// Include body in JSON output.
    #[arg(long)]
    pub full: bool,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// ID of the ish to update.
    pub id: String,
    /// Set the status.
    #[arg(short = 's', long = "status")]
    pub status: Option<String>,
    /// Set the ish type.
    #[arg(short = 't', long = "type")]
    pub ish_type: Option<String>,
    /// Set the priority.
    #[arg(short = 'p', long = "priority")]
    pub priority: Option<String>,
    /// Set the title.
    #[arg(long = "title")]
    pub title: Option<String>,
    /// Replace the full body; use `-` to read from stdin.
    #[arg(
        short = 'd',
        long = "body",
        conflicts_with_all = ["body_file", "body_replace_old", "body_append"]
    )]
    pub body: Option<String>,
    /// Read the full body from a file.
    #[arg(
        long = "body-file",
        conflicts_with_all = ["body", "body_replace_old", "body_append"]
    )]
    pub body_file: Option<String>,
    /// Replace this exact body text.
    #[arg(long = "body-replace-old", requires = "body_replace_new")]
    pub body_replace_old: Option<String>,
    /// Replacement text for `--body-replace-old`.
    #[arg(long = "body-replace-new", requires = "body_replace_old")]
    pub body_replace_new: Option<String>,
    /// Append text to the body; use `-` to read from stdin.
    #[arg(long = "body-append", conflicts_with_all = ["body", "body_file"])]
    pub body_append: Option<String>,
    /// Set the parent ish ID.
    #[arg(long = "parent", conflicts_with = "remove_parent")]
    pub parent: Option<String>,
    /// Remove the current parent relationship.
    #[arg(long = "remove-parent", conflicts_with = "parent")]
    pub remove_parent: bool,
    /// Add a blocking relationship. May be repeated.
    #[arg(long = "blocking")]
    pub blocking: Vec<String>,
    /// Remove a blocking relationship. May be repeated.
    #[arg(long = "remove-blocking")]
    pub remove_blocking: Vec<String>,
    /// Add a blocked-by relationship. May be repeated.
    #[arg(long = "blocked-by")]
    pub blocked_by: Vec<String>,
    /// Remove a blocked-by relationship. May be repeated.
    #[arg(long = "remove-blocked-by")]
    pub remove_blocked_by: Vec<String>,
    /// Add a tag. May be repeated.
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    /// Remove a tag. May be repeated.
    #[arg(long = "remove-tag")]
    pub remove_tags: Vec<String>,
    /// Require the current ETag to match before updating.
    #[arg(long = "if-match")]
    pub if_match: Option<String>,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum ListSortArg {
    Created,
    Updated,
    Status,
    Priority,
    Id,
}

impl ListSortArg {
    pub fn into_sort_mode(self) -> SortMode {
        match self {
            Self::Created => SortMode::Created,
            Self::Updated => SortMode::Updated,
            Self::Status => SortMode::Status,
            Self::Priority => SortMode::Priority,
            Self::Id => SortMode::Id,
        }
    }
}

#[derive(Args)]
pub struct ShowArgs {
    /// IDs of the ishes to display.
    #[arg(required = true)]
    pub ids: Vec<String>,
    /// Print the raw markdown file content.
    #[arg(long, conflicts_with_all = ["body_only", "etag_only"])]
    pub raw: bool,
    /// Print only the markdown body.
    #[arg(long, conflicts_with_all = ["raw", "etag_only"])]
    pub body_only: bool,
    /// Print only the current ETag.
    #[arg(long, conflicts_with_all = ["raw", "body_only"])]
    pub etag_only: bool,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// IDs of the ishes to delete.
    #[arg(required = true)]
    pub ids: Vec<String>,
    /// Skip the confirmation prompt.
    #[arg(short = 'f', long = "force")]
    pub force: bool,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    #[test]
    fn cli_parses_delete_alias() {
        let cli = crate::cli::Cli::try_parse_from(["ish", "rm", "target"])
            .expect("delete alias should parse successfully");

        match cli.command {
            crate::cli::Commands::Delete(args) => {
                assert_eq!(args.ids, vec!["target".to_string()]);
                assert!(!args.force);
            }
            _ => panic!("expected delete command"),
        }
    }

    #[test]
    fn cli_parses_list_alias() {
        let cli = crate::cli::Cli::try_parse_from(["ish", "ls", "--ready"])
            .expect("list alias should parse successfully");

        match cli.command {
            crate::cli::Commands::List(args) => {
                assert!(args.ready);
                assert!(!args.quiet);
            }
            _ => panic!("expected list command"),
        }
    }

    #[test]
    fn cli_parses_show_flags() {
        let cli = crate::cli::Cli::try_parse_from(["ish", "show", "abcd", "efgh", "--body-only"])
            .expect("show command should parse successfully");

        match cli.command {
            crate::cli::Commands::Show(args) => {
                assert_eq!(args.ids, vec!["abcd".to_string(), "efgh".to_string()]);
                assert!(args.body_only);
                assert!(!args.raw);
                assert!(!args.etag_only);
            }
            _ => panic!("expected show command"),
        }
    }

    #[test]
    fn cli_without_subcommand_shows_help() {
        let error = match crate::cli::Cli::try_parse_from(["ish"]) {
            Ok(_) => panic!("cli should require a subcommand"),
            Err(error) => error,
        };

        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        );
        let rendered = error.to_string();
        assert!(rendered.contains("Usage: ish [OPTIONS] <COMMAND>"));
        assert!(rendered.contains("Commands:"));
    }
}
