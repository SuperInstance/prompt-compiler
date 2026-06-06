use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported types for template variables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "options")]
pub enum VariableType {
    String,
    Number,
    Enum(Vec<String>),
}

impl VariableType {
    /// Validate that a value matches this type.
    pub fn validate(&self, value: &str) -> Result<(), String> {
        match self {
            VariableType::String => Ok(()),
            VariableType::Number => {
                if value.parse::<f64>().is_err() {
                    Err(format!("Expected number, got: {value}"))
                } else {
                    Ok(())
                }
            }
            VariableType::Enum(variants) => {
                if variants.contains(&value.to_string()) {
                    Ok(())
                } else {
                    Err(format!(
                        "Value '{value}' not in allowed values: {:?}",
                        variants
                    ))
                }
            }
        }
    }
}

/// A single variable definition within a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub var_type: VariableType,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// A prompt template with `{{variable}}` placeholders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub template: String,
    pub variables: HashMap<String, Variable>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl PromptTemplate {
    /// Create a new template.
    pub fn new(id: impl Into<String>, name: impl Into<String>, template: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            template: template.into(),
            variables: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a variable definition to this template.
    pub fn with_variable(mut self, var: Variable) -> Self {
        self.variables.insert(var.name.clone(), var);
        self
    }

    /// Extract all placeholder names from the template string.
    pub fn placeholders(&self) -> Vec<String> {
        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        re.captures_iter(&self.template)
            .map(|c| c[1].to_string())
            .collect()
    }

    /// Render the template with the given variable values.
    pub fn render(&self, values: &HashMap<String, String>) -> Result<String, RenderError> {
        let mut result = self.template.clone();

        // Check for missing variables (no default)
        for name in &self.placeholders() {
            if !values.contains_key(name) {
                if let Some(var) = self.variables.get(name) {
                    if var.default.is_none() {
                        return Err(RenderError::MissingVariable(name.clone()));
                    }
                }
                // If the variable isn't defined at all and not provided, error
                if !self.variables.contains_key(name) && !values.contains_key(name) {
                    return Err(RenderError::MissingVariable(name.clone()));
                }
            }
        }

        // Validate types
        for (name, value) in values {
            if let Some(var) = self.variables.get(name) {
                var.var_type.validate(value).map_err(|e| RenderError::TypeError {
                    variable: name.clone(),
                    message: e,
                })?;
            }
        }

        // Replace placeholders
        for (name, value) in values {
            result = result.replace(&format!("{{{{{}}}}}", name), value);
        }

        // Fill defaults for unset variables
        for name in &self.placeholders() {
            if !values.contains_key(name) {
                if let Some(var) = self.variables.get(name) {
                    if let Some(ref default) = var.default {
                        result = result.replace(&format!("{{{{{}}}}}", name), default);
                    }
                }
            }
        }

        Ok(result)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Missing variable: {0}")]
    MissingVariable(String),
    #[error("Type error for variable '{variable}': {message}")]
    TypeError { variable: String, message: String },
}
