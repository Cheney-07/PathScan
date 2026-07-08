use crate::error::{PathScanError, Result};
use crate::types::{BuiltinSize, WordlistConfig};
use std::path::PathBuf;

pub struct Wordlist {
    entries: Vec<String>,
}

impl Wordlist {
    pub fn load(config: WordlistConfig, builtin_dir: Option<PathBuf>) -> Result<Self> {
        let mut entries = Vec::new();
        Self::load_config(config, &mut entries, builtin_dir)?;
        Ok(Self { entries })
    }

    fn load_config(
        config: WordlistConfig,
        entries: &mut Vec<String>,
        builtin_dir: Option<PathBuf>,
    ) -> Result<()> {
        match config {
            WordlistConfig::Builtin(size) => {
                let dir = builtin_dir.unwrap_or_else(|| PathBuf::from("wordlists"));
                let filename = match size {
                    BuiltinSize::Small => "small.txt",
                    BuiltinSize::Medium => "medium.txt",
                    BuiltinSize::Large => "large.txt",
                };
                let path = dir.join(filename);
                let content = std::fs::read_to_string(&path).map_err(|_| {
                    PathScanError::WordlistNotFound(path.display().to_string())
                })?;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        entries.push(trimmed.to_string());
                    }
                }
            }
            WordlistConfig::External(filepath) => {
                let content = std::fs::read_to_string(&filepath).map_err(|_| {
                    PathScanError::WordlistNotFound(filepath.clone())
                })?;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        entries.push(trimmed.to_string());
                    }
                }
            }
            WordlistConfig::Multiple(configs) => {
                for cfg in configs {
                    Self::load_config(cfg, entries, builtin_dir.clone())?;
                }
            }
        }
        Ok(())
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    pub fn deduplicate(&mut self) {
        let mut seen = std::collections::HashSet::new();
        self.entries.retain(|e| seen.insert(e.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builtin_dir() -> PathBuf {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        PathBuf::from(&manifest)
            .parent().unwrap().parent().unwrap()
            .join("wordlists")
    }

    #[test]
    fn test_load_builtin_small() {
        let config = WordlistConfig::Builtin(BuiltinSize::Small);
        let wl = Wordlist::load(config, Some(builtin_dir())).unwrap();
        assert!(!wl.entries().is_empty());
        assert!(wl.entries().contains(&"admin".to_string()));
    }

    #[test]
    fn test_deduplicate() {
        let mut wl = Wordlist { entries: vec!["a".into(), "b".into(), "a".into()] };
        wl.deduplicate();
        assert_eq!(wl.entries().len(), 2);
    }

    #[test]
    fn test_skip_comments_and_blanks() {
        let config = WordlistConfig::Builtin(BuiltinSize::Small);
        let wl = Wordlist::load(config, Some(builtin_dir())).unwrap();
        assert!(wl.entries().iter().all(|e| !e.is_empty() && !e.starts_with('#')));
    }
}
