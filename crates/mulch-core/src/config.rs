use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{MulchError, Result};
use crate::types::MulchConfig;

const MULCH_DIR: &str = ".mulch";
const CONFIG_FILE: &str = "mulch.config.yaml";
const EXPERTISE_DIR: &str = "expertise";

pub const GITATTRIBUTES_LINE: &str = ".mulch/expertise/*.jsonl merge=union";

pub const MULCH_README: &str = r#"# .mulch/

This directory is managed by [mulch](https://github.com/jayminwest/mulch) — a structured expertise layer for coding agents.

## Key Commands

- `mulch init`      — Initialize a .mulch directory
- `mulch add`       — Add a new domain
- `mulch record`    — Record an expertise record
- `mulch edit`      — Edit an existing record
- `mulch query`     — Query expertise records
- `mulch prime [domain]` — Output a priming prompt (optionally scoped to one domain)
- `mulch search`   — Search records across domains
- `mulch status`    — Show domain statistics
- `mulch validate`  — Validate all records against the schema
- `mulch prune`     — Remove expired records

## Structure

- `mulch.config.yaml` — Configuration file
- `expertise/`        — JSONL files, one per domain
"#;

pub fn get_mulch_dir(cwd: &Path) -> PathBuf {
    cwd.join(MULCH_DIR)
}

pub fn get_config_path(cwd: &Path) -> PathBuf {
    get_mulch_dir(cwd).join(CONFIG_FILE)
}

pub fn get_expertise_dir(cwd: &Path) -> PathBuf {
    get_mulch_dir(cwd).join(EXPERTISE_DIR)
}

pub fn get_expertise_path(domain: &str, cwd: &Path) -> Result<PathBuf> {
    validate_domain_name(domain)?;
    Ok(get_expertise_dir(cwd).join(format!("{domain}.jsonl")))
}

pub fn validate_domain_name(domain: &str) -> Result<()> {
    let re = regex::Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9_-]*$").unwrap();
    if !re.is_match(domain) {
        return Err(MulchError::InvalidDomainName(domain.to_string()));
    }
    Ok(())
}

pub fn read_config(cwd: &Path) -> Result<MulchConfig> {
    let config_path = get_config_path(cwd);
    let content = fs::read_to_string(&config_path)?;
    let config: MulchConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

pub fn write_config(config: &MulchConfig, cwd: &Path) -> Result<()> {
    let config_path = get_config_path(cwd);
    let content = serde_yaml::to_string(config)?;
    fs::write(&config_path, content)?;
    Ok(())
}

pub fn ensure_mulch_dir(cwd: &Path) -> Result<()> {
    if !get_mulch_dir(cwd).is_dir() {
        return Err(MulchError::NotInitialized);
    }
    Ok(())
}

pub fn ensure_domain_exists(config: &MulchConfig, domain: &str) -> Result<()> {
    if !config.domains.contains(&domain.to_string()) {
        let available = if config.domains.is_empty() {
            "(none)".to_string()
        } else {
            config.domains.join(", ")
        };
        return Err(MulchError::DomainNotFound {
            domain: domain.to_string(),
            available,
        });
    }
    Ok(())
}

pub fn init_mulch_dir(cwd: &Path) -> Result<()> {
    let mulch_dir = get_mulch_dir(cwd);
    let expertise_dir = get_expertise_dir(cwd);
    fs::create_dir_all(&mulch_dir)?;
    fs::create_dir_all(&expertise_dir)?;

    // Only write default config if none exists
    let config_path = get_config_path(cwd);
    if !config_path.exists() {
        write_config(&MulchConfig::default(), cwd)?;
    }

    // Create or append .gitattributes
    let gitattributes_path = cwd.join(".gitattributes");
    let existing = fs::read_to_string(&gitattributes_path).unwrap_or_default();
    if !existing.contains(GITATTRIBUTES_LINE) {
        let separator = if !existing.is_empty() && !existing.ends_with('\n') {
            "\n"
        } else {
            ""
        };
        fs::write(
            &gitattributes_path,
            format!("{existing}{separator}{GITATTRIBUTES_LINE}\n"),
        )?;
    }

    // Create README if missing
    let readme_path = mulch_dir.join("README.md");
    if !readme_path.exists() {
        fs::write(&readme_path, MULCH_README)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_domain_names() {
        assert!(validate_domain_name("rust").is_ok());
        assert!(validate_domain_name("my-domain").is_ok());
        assert!(validate_domain_name("domain_2").is_ok());
        assert!(validate_domain_name("A123").is_ok());
    }

    #[test]
    fn invalid_domain_names() {
        assert!(validate_domain_name("").is_err());
        assert!(validate_domain_name("-starts-with-dash").is_err());
        assert!(validate_domain_name("has spaces").is_err());
        assert!(validate_domain_name("has.dots").is_err());
    }

    #[test]
    fn init_creates_structure() {
        let tmp = tempfile::tempdir().unwrap();
        init_mulch_dir(tmp.path()).unwrap();

        assert!(get_mulch_dir(tmp.path()).is_dir());
        assert!(get_config_path(tmp.path()).exists());
        assert!(get_expertise_dir(tmp.path()).is_dir());
        assert!(tmp.path().join(".gitattributes").exists());

        let config = read_config(tmp.path()).unwrap();
        assert_eq!(config.version, "1");
    }

    #[test]
    fn init_preserves_existing_config() {
        let tmp = tempfile::tempdir().unwrap();
        init_mulch_dir(tmp.path()).unwrap();

        // Modify config
        let mut config = read_config(tmp.path()).unwrap();
        config.domains.push("test".to_string());
        write_config(&config, tmp.path()).unwrap();

        // Re-init should not overwrite
        init_mulch_dir(tmp.path()).unwrap();
        let config = read_config(tmp.path()).unwrap();
        assert_eq!(config.domains, vec!["test"]);
    }
}
