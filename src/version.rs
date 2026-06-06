use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A change to a specific section between versions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SectionChange {
    Added { content: String },
    Removed { content: String },
    Changed { old: String, new: String },
    Unchanged,
}

impl std::fmt::Display for SectionChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Added { content } => write!(f, "+ {content}"),
            Self::Removed { content } => write!(f, "- {content}"),
            Self::Changed { old, new } => write!(f, "- {old}\n+ {new}"),
            Self::Unchanged => write!(f, "  (unchanged)"),
        }
    }
}

/// Diff between two versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiff {
    pub from_version: u32,
    pub to_version: u32,
    pub section_changes: HashMap<String, SectionChange>,
    pub variable_changes: HashMap<String, SectionChange>,
    pub template_changed: bool,
}

impl VersionDiff {
    /// Count the number of actual changes (not unchanged).
    pub fn change_count(&self) -> usize {
        let section_changes = self
            .section_changes
            .values()
            .filter(|c| **c != SectionChange::Unchanged)
            .count();
        let var_changes = self
            .variable_changes
            .values()
            .filter(|c| **c != SectionChange::Unchanged)
            .count();
        section_changes + var_changes + if self.template_changed { 1 } else { 0 }
    }

    /// Returns true if there are no changes.
    pub fn is_empty(&self) -> bool {
        self.change_count() == 0
    }
}

/// A versioned snapshot of a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionedSnapshot {
    version: u32,
    template: String,
    sections: HashMap<String, String>,
    variables: HashMap<String, String>,
    timestamp: String,
}

/// Versioned templates with diff capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVersion {
    pub id: String,
    pub name: String,
    current_version: u32,
    history: Vec<VersionedSnapshot>,
}

impl PromptVersion {
    /// Create a new versioned template.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            current_version: 0,
            history: Vec::new(),
        }
    }

    /// Initialize with a template string. Creates version 1.
    pub fn with_initial_template(mut self, template: &str) -> Self {
        let sections = self.parse_sections(template);
        let snapshot = VersionedSnapshot {
            version: 1,
            template: template.to_string(),
            sections,
            variables: HashMap::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.history.push(snapshot);
        self.current_version = 1;
        self
    }

    /// Get the current version number.
    pub fn current_version(&self) -> u32 {
        self.current_version
    }

    /// Get the current template string.
    pub fn current_template(&self) -> Option<&str> {
        self.history.last().map(|s| s.template.as_str())
    }

    /// Commit a new version of the template.
    pub fn commit(&mut self, template: &str) -> u32 {
        self.current_version += 1;
        let sections = self.parse_sections(template);
        let snapshot = VersionedSnapshot {
            version: self.current_version,
            template: template.to_string(),
            sections,
            variables: HashMap::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.history.push(snapshot);
        self.current_version
    }

    /// Compute the diff between two versions.
    pub fn diff(&self, from: u32, to: u32) -> Result<VersionDiff, String> {
        let from_snap = self
            .history
            .iter()
            .find(|s| s.version == from)
            .ok_or_else(|| format!("Version {from} not found"))?;
        let to_snap = self
            .history
            .iter()
            .find(|s| s.version == to)
            .ok_or_else(|| format!("Version {to} not found"))?;

        // Diff sections
        let mut section_changes = HashMap::new();
        let all_section_keys: std::collections::HashSet<&String> = from_snap
            .sections
            .keys()
            .chain(to_snap.sections.keys())
            .collect();

        for key in all_section_keys {
            let from_val = from_snap.sections.get(key);
            let to_val = to_snap.sections.get(key);
            let change = match (from_val, to_val) {
                (None, Some(new)) => SectionChange::Added {
                    content: new.clone(),
                },
                (Some(old), None) => SectionChange::Removed {
                    content: old.clone(),
                },
                (Some(old), Some(new)) if old == new => SectionChange::Unchanged,
                (Some(old), Some(new)) => SectionChange::Changed {
                    old: old.clone(),
                    new: new.clone(),
                },
                (None, None) => SectionChange::Unchanged,
            };
            section_changes.insert(key.clone(), change);
        }

        // Diff variables
        let mut variable_changes = HashMap::new();
        let all_var_keys: std::collections::HashSet<&String> = from_snap
            .variables
            .keys()
            .chain(to_snap.variables.keys())
            .collect();

        for key in all_var_keys {
            let from_val = from_snap.variables.get(key);
            let to_val = to_snap.variables.get(key);
            let change = match (from_val, to_val) {
                (None, Some(new)) => SectionChange::Added {
                    content: new.clone(),
                },
                (Some(old), None) => SectionChange::Removed {
                    content: old.clone(),
                },
                (Some(old), Some(new)) if old == new => SectionChange::Unchanged,
                (Some(old), Some(new)) => SectionChange::Changed {
                    old: old.clone(),
                    new: new.clone(),
                },
                (None, None) => SectionChange::Unchanged,
            };
            variable_changes.insert(key.clone(), change);
        }

        let template_changed = from_snap.template != to_snap.template;

        Ok(VersionDiff {
            from_version: from,
            to_version: to,
            section_changes,
            variable_changes,
            template_changed,
        })
    }

    /// Roll back to a specific version.
    pub fn rollback(&mut self, version: u32) -> Result<(), String> {
        if !self.history.iter().any(|s| s.version == version) {
            return Err(format!("Version {version} not found"));
        }
        // Keep history up to and including the target version
        self.history.retain(|s| s.version <= version);
        self.current_version = version;
        Ok(())
    }

    /// List all versions.
    pub fn list_versions(&self) -> Vec<u32> {
        self.history.iter().map(|s| s.version).collect()
    }

    /// Parse sections from a template (split by ## headings).
    fn parse_sections(&self, template: &str) -> HashMap<String, String> {
        let mut sections = HashMap::new();
        let mut current_section = "header".to_string();
        let mut current_content = String::new();

        for line in template.lines() {
            if let Some(heading) = line.strip_prefix("## ") {
                // Save previous section
                if !current_content.trim().is_empty() {
                    sections.insert(current_section, current_content.trim().to_string());
                }
                current_section = heading.trim().to_string();
                current_content = String::new();
            } else {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        if !current_content.trim().is_empty() {
            sections.insert(current_section, current_content.trim().to_string());
        }

        sections
    }
}
