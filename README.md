# prompt-compiler

A Rust library for compiling, composing, validating, and versioning prompt templates for LLM workflows.

## Features

- **`PromptTemplate`** — Templates with `{{variable}}` placeholders and typed variables (string, number, enum)
- **`PromptComposer`** — Compose multiple templates into a pipeline (parallel or chained)
- **`TokenBudget`** — Estimate token counts, enforce limits, and auto-truncate
- **`PromptValidator`** — Catch missing variables, circular references, empty sections, and excessive length
- **`PromptVersion`** — Versioned templates with diffs between versions (added/removed/changed sections)
- **`PromptCompiler`** — Compile a template tree into a final prompt string with all variables filled

## Installation

```toml
[dependencies]
prompt-compiler = "0.1.0"
```

## Quick Start

### Basic Template

```rust
use prompt_compiler::*;
use std::collections::HashMap;

let template = PromptTemplate::new("greeting", "Greeting", "Hello, {{name}}! You are {{age}} years old.")
    .with_variable(Variable {
        name: "name".into(),
        var_type: VariableType::String,
        default: None,
        description: Some("User's name".into()),
    })
    .with_variable(Variable {
        name: "age".into(),
        var_type: VariableType::Number,
        default: None,
        description: Some("User's age".into()),
    });

let mut values = HashMap::new();
values.insert("name".into(), "Alice".into());
values.insert("age".into(), "30".into());

let result = template.render(&values).unwrap();
assert_eq!(result, "Hello, Alice! You are 30 years old.");
```

### Enum Validation

```rust
let template = PromptTemplate::new("mode", "Mode Selector", "Running in {{mode}} mode.")
    .with_variable(Variable {
        name: "mode".into(),
        var_type: VariableType::Enum(vec!["fast".into(), "slow".into(), "balanced".into()]),
        default: Some("balanced".into()),
        description: None,
    });

// Valid value
let mut values = HashMap::new();
values.insert("mode".into(), "fast".into());
assert_eq!(template.render(&values).unwrap(), "Running in fast mode.");

// Invalid value — returns error
values.insert("mode".into(), "turbo".into());
assert!(template.render(&values).is_err());
```

### Composing a Pipeline

```rust
let system = PromptTemplate::new("sys", "System", "You are a {{role}}.");
let task = PromptTemplate::new("task", "Task", "Your task: {{task}}.");
let constraints = PromptTemplate::new("con", "Constraints", "Keep responses under {{limit}} words.");

let mut values = HashMap::new();
values.insert("role".into(), "helpful coding assistant".into());
values.insert("task".into(), "write a binary search".into());
values.insert("limit".into(), "200".into());

let prompt = PromptCompiler::new()
    .add_template(system)
    .add_template(task)
    .add_template(constraints)
    .set_variables(values)
    .compile()
    .unwrap();
```

### Chained Composition

In chained mode, each stage's output is available as `{{previous_output}}` for the next:

```rust
let summarize = PromptTemplate::new("s1", "Summarize", "Summarize this: {{topic}}");
let expand = PromptTemplate::new("s2", "Expand", "Now elaborate on: {{previous_output}}");

let mut values = HashMap::new();
values.insert("topic".into(), "Rust programming language".into());

let result = PromptCompiler::new()
    .add_template(summarize)
    .add_template(expand)
    .set_variables(values)
    .compile_chained()
    .unwrap();
```

### Token Budget

```rust
let budget = TokenBudget::new(4096);

let text = "A very long prompt...";
assert!(budget.is_within_budget(text));

let remaining = budget.remaining(text);
println!("{remaining} tokens remaining");

// Auto-truncate to fit
let truncated = budget.truncate_to_budget(text);
```

### Validation

```rust
let validator = PromptValidator::new()
    .with_max_length(50_000);

let errors = validator.validate_template(&my_template);
for error in &errors {
    println!("Validation issue: {error}");
}

// Quick check
if validator.is_valid(&my_template) {
    println!("Template looks good!");
}
```

### Versioning & Diff

```rust
let mut versioned = PromptVersion::new("my-prompt", "My Prompt")
    .with_initial_template("## System\nYou are an AI.\n## Task\nAnswer questions.");

// Evolve the template
versioned.commit("## System\nYou are an expert AI assistant.\n## Task\nAnswer questions precisely.\n## Tone\nBe friendly.");

// See what changed
let diff = versioned.diff(1, 2).unwrap();
println!("Changed sections: {:?}", diff.section_changes.keys().collect::<Vec<_>>());
println!("Total changes: {}", diff.change_count());

// Roll back if needed
versioned.rollback(1).unwrap();
assert_eq!(versioned.current_version(), 1);
```

## Architecture

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Template   │────▶│   Compiler   │────▶│  Final Prompt │
│  (typed vars)│     │  (pipeline)  │     │   (string)    │
└──────────────┘     └──────┬───────┘     └──────────────┘
                            │
                   ┌────────┼────────┐
                   ▼        ▼        ▼
             ┌─────────┐ ┌────────┐ ┌─────────┐
             │Composer │ │Validator│ │  Budget │
             │(stages) │ │(checks)│ │(tokens) │
             └─────────┘ └────────┘ └─────────┘
                            │
                            ▼
                     ┌────────────┐
                     │  Version   │
                     │  (history) │
                     └────────────┘
```

## License

MIT
