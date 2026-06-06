//! # Prompt Compiler
//!
//! A Rust library for compiling, composing, validating, and versioning
//! prompt templates for LLM workflows.

pub mod compiler;
pub mod composer;
pub mod template;
pub mod token_budget;
pub mod validator;
pub mod version;

pub use compiler::{PromptCompiler, CompileError};
pub use composer::{PromptComposer, CompositionError, PipelineStage};
pub use template::{PromptTemplate, VariableType, Variable};
pub use token_budget::{TokenBudget, TokenError};
pub use validator::{PromptValidator, ValidationError};
pub use version::{PromptVersion, VersionDiff, SectionChange};
