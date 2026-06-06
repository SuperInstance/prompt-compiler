use crate::composer::PromptComposer;
use crate::template::PromptTemplate;
use crate::token_budget::TokenBudget;
use std::collections::HashMap;

/// Error from compilation operations.
#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("Missing variable: {0}")]
    MissingVariable(String),
    #[error("Type error: {0}")]
    TypeError(String),
    #[error("Token budget exceeded: {0}")]
    TokenBudget(String),
    #[error("Composition error: {0}")]
    Composition(#[from] crate::composer::CompositionError),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Empty compiler: no templates or stages configured")]
    Empty,
}

/// The main compiler that brings everything together.
///
/// Compiles a template tree into a final prompt string with all variables filled.
#[derive(Debug)]
pub struct PromptCompiler {
    templates: Vec<PromptTemplate>,
    variables: HashMap<String, String>,
    token_budget: Option<TokenBudget>,
    separator: String,
    strict_validation: bool,
}

impl PromptCompiler {
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
            variables: HashMap::new(),
            token_budget: None,
            separator: "\n\n".to_string(),
            strict_validation: false,
        }
    }

    /// Add a template to the compilation.
    pub fn add_template(mut self, template: PromptTemplate) -> Self {
        self.templates.push(template);
        self
    }

    /// Set a variable value.
    pub fn set_variable(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(name.into(), value.into());
        self
    }

    /// Set multiple variables at once.
    pub fn set_variables(mut self, vars: HashMap<String, String>) -> Self {
        self.variables.extend(vars);
        self
    }

    /// Set a token budget.
    pub fn with_token_budget(mut self, budget: TokenBudget) -> Self {
        self.token_budget = Some(budget);
        self
    }

    /// Set the separator between compiled template outputs.
    pub fn with_separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    /// Enable strict validation: all placeholders must have values or defaults.
    pub fn strict(mut self) -> Self {
        self.strict_validation = true;
        self
    }

    /// Compile all templates into a single prompt string.
    pub fn compile(&self) -> Result<String, CompileError> {
        if self.templates.is_empty() {
            return Err(CompileError::Empty);
        }

        // Validate if strict mode
        if self.strict_validation {
            for template in &self.templates {
                for ph in template.placeholders() {
                    if !self.variables.contains_key(&ph) {
                        if let Some(var) = template.variables.get(&ph) {
                            if var.default.is_none() {
                                return Err(CompileError::Validation(format!(
                                    "Template '{}': missing required variable '{}'",
                                    template.id, ph
                                )));
                            }
                        } else {
                            return Err(CompileError::Validation(format!(
                                "Template '{}': undefined variable '{}'",
                                template.id, ph
                            )));
                        }
                    }
                }
            }
        }

        // Render each template
        let mut outputs = Vec::new();
        for template in &self.templates {
            let rendered = template
                .render(&self.variables)
                .map_err(|e| CompileError::TypeError(e.to_string()))?;
            outputs.push(rendered);
        }

        let mut result = outputs.join(&self.separator);

        // Enforce token budget
        if let Some(ref budget) = self.token_budget {
            if !budget.is_within_budget(&result) {
                result = budget.truncate_to_budget(&result);
            }
        }

        Ok(result)
    }

    /// Compile using the composer for pipeline composition.
    pub fn compile_pipelined(&self) -> Result<String, CompileError> {
        if self.templates.is_empty() {
            return Err(CompileError::Empty);
        }

        let mut composer = PromptComposer::new().with_separator(&self.separator);

        if let Some(ref budget) = self.token_budget {
            composer = composer.with_token_budget(budget.clone());
        }

        for template in &self.templates {
            composer = composer.add_stage(template.clone());
        }

        composer.compose(&self.variables).map_err(CompileError::Composition)
    }

    /// Compile in chained mode where each stage's output feeds the next.
    pub fn compile_chained(&self) -> Result<String, CompileError> {
        if self.templates.is_empty() {
            return Err(CompileError::Empty);
        }

        let mut composer = PromptComposer::new();

        if let Some(ref budget) = self.token_budget {
            composer = composer.with_token_budget(budget.clone());
        }

        for template in &self.templates {
            composer = composer.add_stage(template.clone());
        }

        composer
            .compose_chained(&self.variables)
            .map_err(CompileError::Composition)
    }
}

impl Default for PromptCompiler {
    fn default() -> Self {
        Self::new()
    }
}
