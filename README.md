# mulch-rs

[![CI](https://img.shields.io/github/actions/workflow/status/fwindolf/mulch-rs/ci.yml?branch=main)](https://github.com/fwindolf/mulch-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Rust port of [mulch](https://github.com/jayminwest/mulch) — structured expertise files that accumulate over time, live in git, and work with any AI coding agent.

Agents start every session from zero. The pattern your agent discovered yesterday is forgotten today. Mulch fixes this: agents call `mulch record` to write learnings, and `mulch query` to read them. Expertise compounds across sessions, domains, and teammates.

**Mulch is a passive layer.** It does not contain an LLM. Agents use Mulch — Mulch does not use agents.

## Install

### From GitHub Releases

Download the latest binary for your platform from [Releases](https://github.com/fwindolf/mulch-rs/releases).

### From Source

```bash
cargo install --git https://github.com/fwindolf/mulch-rs mulch
```

## Quick Start

```bash
mulch init                                            # Create .mulch/ in your project
mulch add database                                    # Add a domain
mulch record database --type convention "Use WAL mode for SQLite"
mulch record database --type failure \
  --description "VACUUM inside a transaction causes silent corruption" \
  --resolution "Always run VACUUM outside transaction boundaries"
mulch query database                                  # See accumulated expertise
mulch prime                                           # Get full context for agent injection
mulch prime database                                  # Get context for one domain only
```

## How It Works

```
1. mulch init               -> Creates .mulch/ with domain JSONL files
2. Agent reads expertise     -> Grounded in everything the project has learned
3. Agent does work           -> Normal task execution
4. Agent records insights    -> Before finishing, writes learnings back to .mulch/
5. git push                  -> Teammates' agents get smarter too
```

## What's in `.mulch/`

```
.mulch/
├── expertise/
│   ├── database.jsonl        # All database knowledge
│   ├── api.jsonl             # One JSONL file per domain
│   └── testing.jsonl         # Each line is a typed, structured record
└── mulch.config.yaml         # Config: domains, governance settings
```

Everything is git-tracked. Clone a repo and your agents immediately have the project's accumulated expertise.

## CLI Reference

| Command | Description |
|---------|-------------|
| `mulch init` | Initialize `.mulch/` in the current project |
| `mulch add <domain>` | Add a new expertise domain |
| `mulch record <domain> --type <type>` | Record an expertise record (`--tags`, `--force`, `--relates-to`, `--supersedes`, `--batch`, `--stdin`, `--dry-run`, `--evidence-bead`) |
| `mulch edit <domain> <id>` | Edit an existing record by ID or prefix |
| `mulch delete <domain> <id>` | Delete a record by ID or prefix |
| `mulch query [domain]` | Query expertise (`--all`, `--classification`, `--file`, `--outcome-status`, `--sort-by-score`) |
| `mulch prime [domains...]` | Output AI-optimized expertise context (`--budget`, `--no-limit`, `--context`, `--files`, `--exclude-domain`, `--format`, `--export`) |
| `mulch search [query]` | Search records across domains with BM25 ranking (`--domain`, `--type`, `--tag`, `--classification`, `--file`, `--sort-by-score`) |
| `mulch compact [domain]` | Analyze compaction candidates (`--auto`, `--dry-run`) |
| `mulch diff [ref]` | Show expertise changes between git refs |
| `mulch status` | Show expertise freshness and counts |
| `mulch validate` | Schema validation across all files |
| `mulch doctor` | Run health checks (`--fix` to auto-fix) |
| `mulch setup [provider]` | Install provider-specific hooks (claude, cursor, codex, gemini, windsurf, aider) |
| `mulch onboard` | Write onboarding content to agent instruction file (`--agents`, `--claude`, `--copilot`, `--codex`, `--opencode`, `--check`, `--remove`) |
| `mulch prune` | Remove stale tactical/observational entries |
| `mulch ready` | Show recently added or updated records (`--since`, `--domain`, `--limit`) |
| `mulch sync` | Validate, stage, and commit `.mulch/` changes |
| `mulch learn` | Show changed files and suggest domains for recording |

All commands support `--json` for structured JSON output.

## Record Types

| Type | Required Fields | Use Case |
|------|----------------|----------|
| `convention` | content | "Use WAL mode for SQLite connections" |
| `pattern` | name, description | Named patterns with optional file references |
| `failure` | description, resolution | What went wrong and how to avoid it |
| `decision` | title, rationale | Architectural decisions and their reasoning |
| `reference` | name, description | Key files, endpoints, or resources worth remembering |
| `guide` | name, description | Step-by-step procedures for recurring tasks |

All records support optional `--classification` (foundational / tactical / observational), evidence flags (`--evidence-commit`, `--evidence-issue`, `--evidence-file`), `--tags`, `--relates-to`, `--supersedes` for linking, and `--outcome-status` (success/failure/partial) for tracking application results.

## Example Output

```
$ mulch query database

## database (6 records, updated 2h ago)

### Conventions
- [mx-a1b2c3] Use WAL mode for all SQLite connections

### Known Failures
- [mx-d4e5f6] VACUUM inside a transaction causes silent corruption
  -> Always run VACUUM outside transaction boundaries

### Decisions
- [mx-789abc] **SQLite over PostgreSQL**: Local-only product, no network dependency acceptable
```

## Design Principles

- **Zero LLM dependency** -- Mulch makes no LLM calls. Quality equals agent quality.
- **Provider-agnostic** -- Any agent with bash access can call the CLI.
- **Git-native** -- Everything lives in `.mulch/`, tracked in version control.
- **Append-only JSONL** -- Zero merge conflicts, trivial schema validation.
- **Storage != Delivery** -- JSONL on disk, optimized markdown/XML for agents.
- **Format-compatible** -- Reads and writes the same `.mulch/` directory as the [TypeScript version](https://github.com/jayminwest/mulch).

## Concurrency & Multi-Agent Safety

- **Advisory file locking** -- Write commands acquire a `.lock` file before modifying any JSONL file. Retries every 50ms for up to 5 seconds; stale locks (>30s) are auto-removed.
- **Atomic writes** -- All JSONL mutations write to a temp file first, then atomically rename into place.
- **Git merge strategy** -- `mulch init` sets `merge=union` in `.gitattributes` so parallel branches append-merge without conflicts.

## Architecture

```
mulch-rs/
├── crates/
│   ├── mulch-core/    # Library: types, storage, search, scoring, formatting
│   └── mulch/         # Binary: CLI (20 commands)
```

- **mulch-core**: Types (serde tagged enum for 6 record types), JSONL storage with atomic writes, BM25 full-text search, confirmation scoring, token budgeting, output formatting (markdown/XML/plain), git integration, advisory file locking.
- **mulch**: Clap-derived CLI with 20 subcommands, JSON output mode, colored terminal output.

## Tests

132 tests (42 unit + 90 integration) covering all commands, record types, CRUD lifecycle, search, validation, formatting, error handling, and multi-domain workflows.

```bash
cargo test              # Run all tests
cargo test --test integration  # Integration tests only
```

## Attribution

This is a Rust port of [mulch](https://github.com/jayminwest/mulch) by [Jaymin West](https://github.com/jayminwest).

## License

MIT
