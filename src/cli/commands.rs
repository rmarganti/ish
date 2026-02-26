use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::domain::{Issue, ListIssue, Status};
use crate::storage::{IssueRepository, JSONLRepository};

#[derive(Parser)]
#[command(name = "ish")]
#[command(about = "A simple terminal-based issue tracker", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub db_path: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    Add {
        title: String,

        #[arg(short, long)]
        body: Option<String>,

        #[arg(short, long)]
        context: Option<String>,

        #[arg(short, long)]
        parent: Option<String>,
    },
    List {
        #[arg(short, long)]
        status: Option<String>,

        #[arg(short, long)]
        parent: Option<String>,
    },
    Next,
    Start {
        id: String,
    },
    Finish {
        id: String,
    },
    Edit {
        id: String,

        #[arg(short, long)]
        title: Option<String>,

        #[arg(short, long)]
        body: Option<String>,

        #[arg(short, long)]
        context: Option<String>,

        #[arg(short, long)]
        sort: Option<i32>,
    },
    Delete {
        id: String,
    },
    Show {
        id: String,
    },
}

fn get_db_path(cli_db_path: Option<PathBuf>) -> PathBuf {
    if let Some(path) = cli_db_path {
        return path;
    }

    PathBuf::from(".local/issues.jsonl")
}

pub fn run_cli(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = get_db_path(cli.db_path);
    let repo = JSONLRepository::new(db_path)?;

    match cli.command {
        Commands::Add {
            title,
            body,
            context,
            parent,
        } => cmd_add(&repo, title, body, context, parent),
        Commands::List { status, parent } => cmd_list(&repo, status, parent),
        Commands::Next => cmd_next(&repo),
        Commands::Start { id } => cmd_start(&repo, &id),
        Commands::Finish { id } => cmd_finish(&repo, &id),
        Commands::Edit {
            id,
            title,
            body,
            context,
            sort,
        } => cmd_edit(&repo, &id, title, body, context, sort),
        Commands::Delete { id } => cmd_delete(&repo, &id),
        Commands::Show { id } => cmd_show(&repo, &id),
    }
}

fn cmd_add(
    repo: &dyn IssueRepository,
    title: String,
    body: Option<String>,
    context: Option<String>,
    parent: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let issue = Issue::new(title, body, context, parent);
    repo.create(&issue)?;
    println!("{}", serde_json::to_string_pretty(&issue)?);
    Ok(())
}

fn cmd_list(
    repo: &dyn IssueRepository,
    status: Option<String>,
    parent: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let issues = match (status, parent) {
        (Some(s), _) => {
            let st = s.parse::<Status>()?;
            repo.get_by_status(st)?
        }
        (_, Some(p)) => repo.get_by_parent(Some(&p))?,
        (_, _) => repo.get_all()?,
    };

    let list_issues: Vec<ListIssue> = issues.into_iter().map(ListIssue::from).collect();
    println!("{}", serde_json::to_string_pretty(&list_issues)?);
    Ok(())
}

fn cmd_next(repo: &dyn IssueRepository) -> Result<(), Box<dyn std::error::Error>> {
    let issue = repo.get_next_todo()?;
    match issue {
        Some(i) => {
            println!("{}", serde_json::to_string_pretty(&vec![i])?);
            Ok(())
        }
        None => Err("No todo issues found".into()),
    }
}

fn cmd_start(repo: &dyn IssueRepository, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut issue = repo.get_by_id(id)?;
    issue.start()?;
    repo.update(&issue)?;
    println!("{}", serde_json::to_string_pretty(&issue)?);
    Ok(())
}

fn cmd_finish(repo: &dyn IssueRepository, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut issue = repo.get_by_id(id)?;
    issue.finish()?;
    repo.update(&issue)?;
    println!("{}", serde_json::to_string_pretty(&issue)?);
    Ok(())
}

fn cmd_edit(
    repo: &dyn IssueRepository,
    id: &str,
    title: Option<String>,
    body: Option<String>,
    context: Option<String>,
    sort: Option<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut issue = repo.get_by_id(id)?;

    if let Some(t) = title {
        issue.update_title(t);
    }
    if body.is_some() || body == Some(String::new()) {
        issue.update_body(body);
    }
    if context.is_some() || context == Some(String::new()) {
        issue.update_context(context);
    }
    if let Some(s) = sort {
        issue.update_sort(s);
    }

    repo.update(&issue)?;
    println!("{}", serde_json::to_string_pretty(&issue)?);
    Ok(())
}

fn cmd_delete(repo: &dyn IssueRepository, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let issue = repo.get_by_id(id)?;
    println!("{}", serde_json::to_string_pretty(&issue)?);
    repo.delete(id)?;
    Ok(())
}

fn cmd_show(repo: &dyn IssueRepository, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let issue = repo.get_by_id(id)?;
    println!("{}", serde_json::to_string_pretty(&issue)?);
    Ok(())
}
