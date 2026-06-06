use crate::template::PromptTemplate;
use crate::composer::PromptComposer;
use std::collections::HashMap;

/// Validation error with details about what went wrong.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    MissingVariable {
        template_id: String,
        variable: String,
    },
    UndefinedVariable {
        template_id: String,
        variable: String,
    },
    EmptySection {
        template_id: String,
        section: String,
    },
    ExcessiveLength {
        template_id: String,
        length: usize,
        max: usize,
    },
    CircularReference {
        template_ids: Vec<String>,
    },
    DuplicateVariable {
        template_id: String,
        variable: String,
    },
    InvalidTemplate {
        template_id: String,
        reason: String,
    },
    EmptyPipeline,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingVariable { template_id, variable } => {
                write!(f, "Template '{template_id}': missing required variable '{variable}'")
            }
            Self::UndefinedVariable { template_id, variable } => {
                write!(f, "Template '{template_id}': placeholder '{variable}' has no variable definition")
            }
            Self::EmptySection { template_id, section } => {
                write!(f, "Template '{template_id}': empty section '{section}'")
            }
            Self::ExcessiveLength { template_id, length, max } => {
                write!(f, "Template '{template_id}': length {length} exceeds max {max}")
            }
            Self::CircularReference { template_ids } => {
                write!(f, "Circular reference detected: {}", template_ids.join(" -> "))
            }
            Self::DuplicateVariable { template_id, variable } => {
                write!(f, "Template '{template_id}': duplicate variable '{variable}'")
            }
            Self::InvalidTemplate { template_id, reason } => {
                write!(f, "Template '{template_id}': invalid template: {reason}")
            }
            Self::EmptyPipeline => write!(f, "Empty pipeline: no stages to validate"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validates templates and compositions for common issues.
#[derive(Debug, Clone)]
pub struct PromptValidator {
    max_template_length: usize,
    max_pipeline_depth: usize,
}

impl PromptValidator {
    pub fn new() -> Self {
        Self {
            max_template_length: 100_000,
            max_pipeline_depth: 50,
        }
    }

    /// Set the maximum allowed template length in characters.
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_template_length = max;
        self
    }

    /// Validate a single template.
    pub fn validate_template(&self, template: &PromptTemplate) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let placeholders = template.placeholders();
        let mut seen_placeholders = std::collections::HashSet::new();

        for ph in &placeholders {
            if !seen_placeholders.insert(ph.clone()) {
                errors.push(ValidationError::DuplicateVariable {
                    template_id: template.id.clone(),
                    variable: ph.clone(),
                });
            }
        }

        // Check for undefined placeholders (in template but not in variables)
        for ph in &placeholders {
            if !template.variables.contains_key(ph) {
                errors.push(ValidationError::UndefinedVariable {
                    template_id: template.id.clone(),
                    variable: ph.clone(),
                });
            }
        }

        // Check for variables defined but not used in template
        for var_name in template.variables.keys() {
            if !placeholders.contains(var_name) {
                // Not necessarily an error, but could indicate a bug
                // We skip this for now
            }
        }

        // Check for empty template
        if template.template.trim().is_empty() {
            errors.push(ValidationError::EmptySection {
                template_id: template.id.clone(),
                section: "template_body".to_string(),
            });
        }

        // Check excessive length
        if template.template.len() > self.max_template_length {
            errors.push(ValidationError::ExcessiveLength {
                template_id: template.id.clone(),
                length: template.template.len(),
                max: self.max_template_length,
            });
        }

        errors
    }

    /// Validate that all required variables are provided.
    pub fn validate_variables(
        &self,
        template: &PromptTemplate,
        values: &HashMap<String, String>,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let placeholders = template.placeholders();

        for ph in &placeholders {
            if !values.contains_key(ph) {
                if let Some(var) = template.variables.get(ph) {
                    if var.default.is_none() {
                        errors.push(ValidationError::MissingVariable {
                            template_id: template.id.clone(),
                            variable: ph.clone(),
                        });
                    }
                }
            }
        }

        errors
    }

    /// Validate an entire pipeline.
    pub fn validate_pipeline(&self, composer: &PromptComposer) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if composer.stage_count() == 0 {
            errors.push(ValidationError::EmptyPipeline);
            return errors;
        }

        if composer.stage_count() > self.max_pipeline_depth {
            errors.push(ValidationError::ExcessiveLength {
                template_id: "pipeline".to_string(),
                length: composer.stage_count(),
                max: self.max_pipeline_depth,
            });
        }

        // Check for circular references (duplicate template IDs in pipeline)
        let _seen_ids: Vec<String> = Vec::new();
        let _all_ids: Vec<String> = Vec::new();
        // Structural checks would go here if stages were exposed

        errors
    }

    /// Quick check: is the template valid with no errors?
    pub fn is_valid(&self, template: &PromptTemplate) -> bool {
        self.validate_template(template).is_empty()
    }
}

impl Default for PromptValidator {
    fn default() -> Self {
        Self::new()
    }
}
