use crate::template::PromptTemplate;
use crate::token_budget::TokenBudget;
use std::collections::HashMap;

/// A stage in the composition pipeline.
#[derive(Debug, Clone)]
pub struct PipelineStage {
    pub template: PromptTemplate,
    pub label: Option<String>,
}

/// Error from composition operations.
#[derive(Debug, thiserror::Error)]
pub enum CompositionError {
    #[error("Empty pipeline: no stages added")]
    EmptyPipeline,
    #[error("Render error at stage '{stage}': {source}")]
    RenderFailed {
        stage: String,
        #[source]
        source: crate::template::RenderError,
    },
    #[error("Token budget exceeded at stage '{stage}': used {used}, max {max}")]
    TokenBudgetExceeded { stage: String, used: usize, max: usize },
    #[error("Circular reference detected: pipeline stage '{stage}' referenced itself")]
    CircularReference { stage: String },
}

/// Composes multiple templates into a pipeline where the output of one
/// feeds into the next.
#[derive(Debug, Clone)]
pub struct PromptComposer {
    stages: Vec<PipelineStage>,
    separator: String,
    token_budget: Option<TokenBudget>,
}

impl PromptComposer {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
            separator: "\n\n".to_string(),
            token_budget: None,
        }
    }

    /// Set the separator between stage outputs.
    pub fn with_separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    /// Set a token budget for the composed output.
    pub fn with_token_budget(mut self, budget: TokenBudget) -> Self {
        self.token_budget = Some(budget);
        self
    }

    /// Add a template as a pipeline stage.
    pub fn add_stage(mut self, template: PromptTemplate) -> Self {
        let label = Some(template.name.clone());
        self.stages.push(PipelineStage { template, label });
        self
    }

    /// Add a stage with an explicit label.
    pub fn add_labeled_stage(mut self, template: PromptTemplate, label: impl Into<String>) -> Self {
        self.stages.push(PipelineStage {
            template,
            label: Some(label.into()),
        });
        self
    }

    /// Get the number of stages.
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Compose all stages into a single prompt string.
    ///
    /// Each stage receives the same variables. Stages are joined with the separator.
    pub fn compose(&self, values: &HashMap<String, String>) -> Result<String, CompositionError> {
        if self.stages.is_empty() {
            return Err(CompositionError::EmptyPipeline);
        }

        let mut outputs = Vec::new();

        for (i, stage) in self.stages.iter().enumerate() {
            let stage_name = stage
                .label
                .clone()
                .unwrap_or_else(|| format!("stage_{i}"));

            let rendered = stage
                .template
                .render(values)
                .map_err(|e| CompositionError::RenderFailed {
                    stage: stage_name,
                    source: e,
                })?;

            outputs.push(rendered);
        }

        let mut result = outputs.join(&self.separator);

        // Enforce token budget
        if let Some(ref budget) = self.token_budget {
            let count = budget.estimate_tokens(&result);
            if count > budget.max_tokens() {
                return Err(CompositionError::TokenBudgetExceeded {
                    stage: "final_output".to_string(),
                    used: count,
                    max: budget.max_tokens(),
                });
            }
            result = budget.truncate_to_budget(&result);
        }

        Ok(result)
    }

    /// Compose stages in chain mode: each stage's output is available as
    /// `{{previous_output}}` for the next stage.
    pub fn compose_chained(&self, initial_values: &HashMap<String, String>) -> Result<String, CompositionError> {
        if self.stages.is_empty() {
            return Err(CompositionError::EmptyPipeline);
        }

        let mut values = initial_values.clone();
        let mut final_output = String::new();

        for (i, stage) in self.stages.iter().enumerate() {
            let stage_name = stage
                .label
                .clone()
                .unwrap_or_else(|| format!("stage_{i}"));

            let rendered = stage
                .template
                .render(&values)
                .map_err(|e| CompositionError::RenderFailed {
                    stage: stage_name,
                    source: e,
                })?;

            // Feed output into next stage
            values.insert("previous_output".to_string(), rendered.clone());
            final_output = rendered;
        }

        // Enforce token budget
        if let Some(ref budget) = self.token_budget {
            let count = budget.estimate_tokens(&final_output);
            if count > budget.max_tokens() {
                return Err(CompositionError::TokenBudgetExceeded {
                    stage: "final_output".to_string(),
                    used: count,
                    max: budget.max_tokens(),
                });
            }
            final_output = budget.truncate_to_budget(&final_output);
        }

        Ok(final_output)
    }
}

impl Default for PromptComposer {
    fn default() -> Self {
        Self::new()
    }
}
