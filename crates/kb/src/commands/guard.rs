use anyhow::Result;
use std::collections::BTreeSet;
use std::io::Read;

use crate::context::RuntimeContext;
use kb_core::{access_log, config};

#[derive(serde::Deserialize)]
struct HookInput {
    #[serde(default)]
    stop_hook_active: bool,
    session_id: String,
}

pub fn run(ctx: &RuntimeContext) -> Result<()> {
    config::ensure_kb_dir(&ctx.cwd)?;

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let hook: HookInput = serde_json::from_str(&input)?;

    // Already in a stop hook → don't block recursively
    if hook.stop_hook_active {
        return Ok(());
    }

    let all_entries = access_log::query_log(
        &ctx.cwd,
        &access_log::AccessLogFilter {
            session_id: Some(hook.session_id.clone()),
            ..Default::default()
        },
    )?;

    // kb wasn't used this session
    if all_entries.is_empty() {
        return Ok(());
    }

    // Check if learn was already called (any tool="learn" entry for this session)
    let has_learn = all_entries.iter().any(|e| e.tool == "learn");
    if has_learn {
        return Ok(());
    }

    // Collect active domains
    let domains: BTreeSet<&str> = all_entries
        .iter()
        .filter_map(|e| e.domain.as_deref())
        .collect();

    let domain_list = if domains.is_empty() {
        String::from("(no specific domains)")
    } else {
        domains.into_iter().collect::<Vec<_>>().join(", ")
    };

    eprintln!("kb was used this session but `kb learn` was not called.");
    eprintln!();
    eprintln!("Active domains: {domain_list}");
    eprintln!();
    eprintln!("Before ending, please:");
    eprintln!("  1. Run `kb learn` to review what changed");
    eprintln!("  2. Run `kb record <domain> --type <type> --description \"...\"` for each insight");
    eprintln!("  3. Run `kb sync` to commit knowledge");
    eprintln!();
    eprintln!("If nothing important came up, that's fine:");
    eprintln!("  kb learn --skip --session {}", hook.session_id);

    std::process::exit(2);
}
