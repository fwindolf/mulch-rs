use anyhow::{Context, Result};
use std::fs;

use crate::cli::OnboardArgs;
use crate::context::RuntimeContext;
use crate::output::*;
use mulch_core::{config, markers, storage};

fn build_expertise_section(ctx: &RuntimeContext, provider: Option<&str>) -> Result<String> {
    let cfg = config::read_config(&ctx.cwd)?;

    let mut lines = vec![
        "## Project Expertise (Mulch)".to_string(),
        String::new(),
        "This project uses [mulch](https://github.com/jayminwest/mulch) to manage structured expertise for coding agents.".to_string(),
        String::new(),
        "### Quick Start".to_string(),
        String::new(),
        "```bash".to_string(),
        "mulch prime          # Load expertise at session start".to_string(),
        "mulch search \"query\" # Find relevant records".to_string(),
        "mulch record <domain> --type <type> --description \"...\"".to_string(),
        "mulch sync           # Commit changes".to_string(),
        "```".to_string(),
        String::new(),
    ];

    // Domain summary
    if !cfg.domains.is_empty() {
        lines.push("### Domains".to_string());
        lines.push(String::new());
        for domain in &cfg.domains {
            let file_path = config::get_expertise_path(domain, &ctx.cwd)?;
            let records = storage::read_expertise_file(&file_path)?;
            lines.push(format!("- **{}**: {} record(s)", domain, records.len()));
        }
        lines.push(String::new());
    }

    // Provider-specific instructions
    if let Some(provider) = provider {
        lines.push(format!("### {} Integration", capitalize(provider)));
        lines.push(String::new());
        match provider {
            "claude" => {
                lines.push("Add to your Claude session: `mulch prime`".to_string());
            }
            "cursor" => {
                lines.push(
                    "Cursor will automatically pick up rules from `.cursor/rules`.".to_string(),
                );
            }
            _ => {
                lines.push(format!(
                    "Run `mulch setup --provider {provider}` to configure."
                ));
            }
        }
        lines.push(String::new());
    }

    Ok(lines.join("\n"))
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

pub fn run(ctx: &RuntimeContext, args: &OnboardArgs) -> Result<()> {
    config::ensure_mulch_dir(&ctx.cwd)?;

    let readme_path = ctx.cwd.join("README.md");
    let section = build_expertise_section(ctx, args.provider.as_deref())?;
    let wrapped = markers::wrap_in_markers(&format!("{section}\n"));

    if readme_path.exists() {
        let content = fs::read_to_string(&readme_path).context("Failed to read README.md")?;

        if args.update && markers::has_marker_section(&content) {
            // Replace existing section
            if let Some(updated) = markers::replace_marker_section(&content, &wrapped) {
                fs::write(&readme_path, updated)?;
                if ctx.json {
                    output_json(&serde_json::json!({
                        "success": true,
                        "command": "onboard",
                        "action": "updated",
                    }));
                } else {
                    print_success("Updated mulch expertise section in README.md.");
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
                }));
            } else {
                println!("Mulch section already exists in README.md. Use --update to replace it.");
            }
            return Ok(());
        }

        // Append section
        let updated = format!("{}\n\n{}\n", content.trim_end(), wrapped);
        fs::write(&readme_path, updated)?;
    } else {
        // Create README.md with section
        let content = format!("# Project\n\n{}\n", wrapped);
        fs::write(&readme_path, content)?;
    }

    if ctx.json {
        output_json(&serde_json::json!({
            "success": true,
            "command": "onboard",
            "action": "created",
        }));
    } else {
        print_success("Added mulch expertise section to README.md.");
    }

    Ok(())
}
