# Ish

Ish: local issue tracking for humans _and_ agents

Ish is a terminal-based issue tracker that stores issues as markdown files with
YAML frontmatter—right inside your repo. No server, no database, no accounts.
Just files you can read, edit, grep, and commit alongside your code.

Every issue (called an "ish") is a `.md` file in a `.ish/` directory. A
built-in `--json` mode and an `ish prime` command that prints an agent-ready
project guide make Ish a natural fit for AI coding agents that need to
discover, create, and update work items programmatically.

## Quick Start

```bash
# Initialize a new ish project in the current directory
ish init

# Create an issue
ish create "Fix the login redirect bug" --type bug --priority high

# List open issues
ish list

# Update an issue
ish update <id> --status in-progress

# Show full details
ish show <id>

# Mark complete and archive
ish update <id> --status completed
ish archive
```

## Features

- **Plain-text storage** — Issues are human-readable markdown files with YAML frontmatter, committed to version control like any other source file.
- **Rich CLI** — Filter, sort, search, and display issues from the terminal with colored output and tree views.
- **Structured JSON output** — Every command supports `--json` for machine-readable output, making it easy to script or integrate with other tools.
- **Agent-first design** — `ish prime` emits a project-aware prompt that teaches an AI agent how to use Ish in your repo. Optimistic concurrency via ETags (`--if-match`) keeps parallel agents from clobbering each other's edits.
- **Hierarchy & dependencies** — Organize work with milestones → epics → features → tasks. Model sequencing with `--blocking` / `--blocked-by` relationships, and query what's ready with `ish list --ready`.
- **Roadmap generation** — `ish roadmap` renders a milestone/epic hierarchy as markdown (or JSON), suitable for pasting into a README or feeding to a planning tool.
- **Link integrity** — `ish check` validates all parent, blocking, and blocked-by references. `ish check --fix` auto-repairs broken links.
- **Archive** — `ish archive` moves completed and scrapped issues out of the active directory so the working set stays clean.

## Issue Model

Each ish has the following metadata:

| Field        | Values                                                  |
| ------------ | ------------------------------------------------------- |
| **Type**     | `milestone`, `epic`, `bug`, `feature`, `task`           |
| **Status**   | `in-progress`, `todo`, `draft`, `completed`, `scrapped` |
| **Priority** | `critical`, `high`, `normal`, `low`, `deferred`         |

Issues also support freeform **tags**, a **parent** reference for hierarchy, and **blocking** / **blocked-by** lists for dependency tracking.

## Configuration

Ish is configured via a `.ish.yml` file in your project root:

```yaml
ish:
    path: .ish # directory where issue files are stored
    prefix: ish # ID prefix (e.g. ish-a1b2)
    id_length: 4 # random suffix length
    default_status: todo
    default_type: task
project:
    name: My Project
```

## Commands

| Command         | Description                                             |
| --------------- | ------------------------------------------------------- |
| `init`          | Create a new `.ish.yml` and initialize the workspace    |
| `create`        | Create a new issue                                      |
| `list` / `ls`   | List issues with filtering, sorting, and search         |
| `update` / `u`  | Update issue metadata or body content                   |
| `show`          | Display full issue details with rendered markdown       |
| `delete` / `rm` | Delete one or more issues                               |
| `archive`       | Move completed/scrapped issues to the archive directory |
| `check`         | Validate configuration and link integrity               |
| `roadmap`       | Generate a milestone/epic hierarchy view                |
| `prime`         | Print an AI-agent guide for the current project         |
| `version`       | Print the current ish version                           |

## For AI Agents

Run `ish prime` to get a project-specific prompt you can feed to an AI agent.
It includes the full command reference, valid types/statuses/priorities, and
working rules for safe concurrent editing.

Use `--json` on any command for structured output. Use `--if-match <etag>` on
updates to protect against concurrent edits—Ish will reject the write if the
issue has changed since you last read it.
