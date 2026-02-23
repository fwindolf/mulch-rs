use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::cli::OnboardArgs;
use crate::context::RuntimeContext;
use crate::output::*;
use mulch_core::{config, markers, storage};

fn build_expertise_section(ctx: &RuntimeContext) -> Result<String> {
    let cfg = config::read_config(&ctx.cwd)?;

    let mut lines = vec![
        "## Mulch".to_string(),
        String::new(),
        "Use Mulch to build knowledge about the project. DO NOT USE MEMORY.md".to_string(),
        "At session start: `mulch prime` (or `mulch prime --files <path>` for targeted context).".to_string(),
        "Before finishing: `mulch learn` → `mulch record <domain> --type <type> --description \"...\"` → `mulch sync`.".to_string(),
        "Types: convention | pattern | failure | decision | reference | guide.".to_string(),
    ];

    // Domain summary
    if !cfg.domains.is_empty() {
        lines.push(String::new());
        lines.push("### Domains".to_string());
        lines.push(String::new());
        for domain in &cfg.domains {
            let file_path = config::get_expertise_path(domain, &ctx.cwd)?;
            let records = storage::read_expertise_file(&file_path)?;
            lines.push(format!("- **{}**: {} record(s)", domain, records.len()));
        }
    }

    Ok(lines.join("\n"))
}

/// Find the target file for onboarding: existing CLAUDE.md or AGENTS.md,
/// falling back to creating AGENTS.md.
fn resolve_target_file(cwd: &std::path::Path) -> PathBuf {
    let claude_md = cwd.join("CLAUDE.md");
    if claude_md.exists() {
        return claude_md;
    }
    cwd.join("AGENTS.md")
}

pub fn run(ctx: &RuntimeContext, args: &OnboardArgs) -> Result<()> {
    config::ensure_mulch_dir(&ctx.cwd)?;

    let target_path = resolve_target_file(&ctx.cwd);
    let target_name = target_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let section = build_expertise_section(ctx)?;
    let wrapped = markers::wrap_in_markers(&format!("{section}\n"));

    if target_path.exists() {
        let content =
            fs::read_to_string(&target_path).context(format!("Failed to read {target_name}"))?;

        if args.update && markers::has_marker_section(&content) {
            // Replace existing section
            if let Some(updated) = markers::replace_marker_section(&content, &wrapped) {
                fs::write(&target_path, updated)?;
                if ctx.json {
                    output_json(&serde_json::json!({
                        "success": true,
                        "command": "onboard",
                        "action": "updated",
                        "file": target_name,
                    }));
                } else {
                    print_success(&format!(
                        "Updated mulch expertise section in {target_name}."
                    ));
                }
                return Ok(());
            }
        }

        if markers::has_marker_section(&content) {
            if ctx.json {
                output_json(&serde_json::json!({
                    "success": true,
                    "command": "onboard",
                    "action": "already_exists",
                    "file": target_name,
                }));
            } else {
                println!(
                    "Mulch section already exists in {target_name}. Use --update to replace it."
                );
            }
            return Ok(());
        }

        // Append section
        let updated = format!("{}\n\n{}\n", content.trim_end(), wrapped);
        fs::write(&target_path, updated)?;
    } else {
        // Create file with section
        fs::write(&target_path, format!("{wrapped}\n"))?;
    }

    if ctx.json {
        output_json(&serde_json::json!({
            "success": true,
            "command": "onboard",
            "action": "created",
            "file": target_name,
        }));
    } else {
        print_success(&format!("Added mulch expertise section to {target_name}."));
    }

    Ok(())
}
