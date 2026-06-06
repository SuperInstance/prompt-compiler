use prompt_compiler::*;
use std::collections::HashMap;

// ─── Template Tests ──────────────────────────────────────────────────────────

#[test]
fn test_template_basic_render() {
    let tpl = PromptTemplate::new("t1", "Test", "Hello, {{name}}!")
        .with_variable(Variable {
            name: "name".into(),
            var_type: VariableType::String,
            default: None,
            description: None,
        });

    let mut values = HashMap::new();
    values.insert("name".into(), "World".into());

    assert_eq!(tpl.render(&values).unwrap(), "Hello, World!");
}

#[test]
fn test_template_missing_variable() {
    let tpl = PromptTemplate::new("t1", "Test", "Hello, {{name}}!");
    let values = HashMap::new();
    assert!(tpl.render(&values).is_err());
}

#[test]
fn test_template_default_value() {
    let tpl = PromptTemplate::new("t1", "Test", "Hello, {{name}}!")
        .with_variable(Variable {
            name: "name".into(),
            var_type: VariableType::String,
            default: Some("friend".into()),
            description: None,
        });

    let values = HashMap::new();
    assert_eq!(tpl.render(&values).unwrap(), "Hello, friend!");
}

#[test]
fn test_template_number_type_validation() {
    let tpl = PromptTemplate::new("t1", "Test", "Count: {{count}}")
        .with_variable(Variable {
            name: "count".into(),
            var_type: VariableType::Number,
            default: None,
            description: None,
        });

    let mut values = HashMap::new();
    values.insert("count".into(), "42".into());
    assert_eq!(tpl.render(&values).unwrap(), "Count: 42");

    values.insert("count".into(), "not_a_number".into());
    assert!(tpl.render(&values).is_err());
}

#[test]
fn test_template_enum_type_validation() {
    let tpl = PromptTemplate::new("t1", "Test", "Mode: {{mode}}")
        .with_variable(Variable {
            name: "mode".into(),
            var_type: VariableType::Enum(vec!["fast".into(), "slow".into(), "balanced".into()]),
            default: None,
            description: None,
        });

    let mut values = HashMap::new();
    values.insert("mode".into(), "fast".into());
    assert_eq!(tpl.render(&values).unwrap(), "Mode: fast");

    values.insert("mode".into(), "turbo".into());
    assert!(tpl.render(&values).is_err());
}

#[test]
fn test_template_multiple_placeholders() {
    let tpl = PromptTemplate::new("t1", "Test", "{{greeting}}, {{name}}! Welcome to {{place}}.");
    let mut values = HashMap::new();
    values.insert("greeting".into(), "Hello".into());
    values.insert("name".into(), "Alice".into());
    values.insert("place".into(), "Wonderland".into());
    assert_eq!(
        tpl.render(&values).unwrap(),
        "Hello, Alice! Welcome to Wonderland."
    );
}

#[test]
fn test_template_placeholders_extraction() {
    let tpl = PromptTemplate::new("t1", "Test", "{{a}} and {{b}} and {{c}} and {{a}}");
    let phs = tpl.placeholders();
    assert_eq!(phs, vec!["a", "b", "c", "a"]);
}

// ─── Token Budget Tests ──────────────────────────────────────────────────────

#[test]
fn test_token_budget_estimation() {
    let budget = TokenBudget::new(100);
    let text = "Hello, this is a test of token estimation.";
    let tokens = budget.estimate_tokens(text);
    assert!(tokens > 0);
    assert!(tokens < 100);
}

#[test]
fn test_token_budget_within_limit() {
    let budget = TokenBudget::new(1000);
    let text = "Short text.";
    assert!(budget.is_within_budget(text));
    assert!(budget.validate(text).is_ok());
}

#[test]
fn test_token_budget_truncation() {
    let budget = TokenBudget::new(5);
    let long_text = "This is a very long text that should definitely be truncated by the token budget system.";
    let truncated = budget.truncate_to_budget(long_text);
    assert!(budget.is_within_budget(&truncated));
    assert!(truncated.len() < long_text.len());
}

#[test]
fn test_token_budget_exceeded_error() {
    let budget = TokenBudget::new(2);
    let text = "This is way too long for a two token budget.";
    assert!(budget.validate(text).is_err());
}

#[test]
fn test_token_budget_remaining() {
    let budget = TokenBudget::new(100);
    let text = "Hello world";
    let remaining = budget.remaining(text);
    assert!(remaining < 100);
    assert!(remaining > 0);
}

#[test]
fn test_token_budget_empty_text() {
    let budget = TokenBudget::new(100);
    assert_eq!(budget.estimate_tokens(""), 0);
    assert!(budget.is_within_budget(""));
}

// ─── Composer Tests ──────────────────────────────────────────────────────────

#[test]
fn test_composer_basic() {
    let t1 = PromptTemplate::new("t1", "Role", "You are a {{role}}.");
    let t2 = PromptTemplate::new("t2", "Task", "Your task is to {{task}}.");

    let mut values = HashMap::new();
    values.insert("role".into(), "helpful assistant".into());
    values.insert("task".into(), "answer questions".into());

    let result = PromptComposer::new()
        .add_stage(t1)
        .add_stage(t2)
        .compose(&values)
        .unwrap();

    assert!(result.contains("helpful assistant"));
    assert!(result.contains("answer questions"));
    assert!(result.contains("\n\n"));
}

#[test]
fn test_composer_empty_pipeline() {
    let values = HashMap::new();
    let result = PromptComposer::new().compose(&values);
    assert!(result.is_err());
}

#[test]
fn test_composer_custom_separator() {
    let t1 = PromptTemplate::new("t1", "A", "Part A");
    let t2 = PromptTemplate::new("t2", "B", "Part B");

    let values = HashMap::new();
    let result = PromptComposer::new()
        .with_separator("---\n")
        .add_stage(t1)
        .add_stage(t2)
        .compose(&values)
        .unwrap();

    assert_eq!(result, "Part A---\nPart B");
}

#[test]
fn test_composer_chained() {
    let t1 = PromptTemplate::new("t1", "First", "Summarize: {{topic}}");
    let t2 = PromptTemplate::new("t2", "Second", "Expand on: {{previous_output}}");

    let mut values = HashMap::new();
    values.insert("topic".into(), "Rust programming".into());

    let result = PromptComposer::new()
        .add_stage(t1)
        .add_stage(t2)
        .compose_chained(&values)
        .unwrap();

    assert!(result.contains("Summarize: Rust programming"));
}

#[test]
fn test_composer_token_budget_enforcement() {
    let t1 = PromptTemplate::new("t1", "Long", "Word ".repeat(200));
    let budget = TokenBudget::new(5);

    let values = HashMap::new();
    let result = PromptComposer::new()
        .with_token_budget(budget)
        .add_stage(t1)
        .compose(&values);

    // Should either error or truncate
    match result {
        Ok(truncated) => assert!(truncated.len() < 1000),
        Err(_) => {} // TokenBudgetExceeded error is also acceptable
    }
}

// ─── Validator Tests ─────────────────────────────────────────────────────────

#[test]
fn test_validator_undefined_placeholder() {
    let tpl = PromptTemplate::new("t1", "Test", "Hello {{name}} and {{age}}!")
        .with_variable(Variable {
            name: "name".into(),
            var_type: VariableType::String,
            default: None,
            description: None,
        });
    // "age" is used in template but not defined as a variable

    let validator = PromptValidator::new();
    let errors = validator.validate_template(&tpl);
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::UndefinedVariable { variable, .. } if variable == "age"
    )));
}

#[test]
fn test_validator_empty_template() {
    let tpl = PromptTemplate::new("t1", "Test", "   ");
    let validator = PromptValidator::new();
    let errors = validator.validate_template(&tpl);
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::EmptySection { .. })));
}

#[test]
fn test_validator_excessive_length() {
    let tpl = PromptTemplate::new("t1", "Test", "x".repeat(200));
    let validator = PromptValidator::new().with_max_length(100);
    let errors = validator.validate_template(&tpl);
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::ExcessiveLength { .. })));
}

#[test]
fn test_validator_valid_template() {
    let tpl = PromptTemplate::new("t1", "Test", "Hello {{name}}!")
        .with_variable(Variable {
            name: "name".into(),
            var_type: VariableType::String,
            default: Some("world".into()),
            description: None,
        });

    let validator = PromptValidator::new();
    assert!(validator.is_valid(&tpl));
}

#[test]
fn test_validator_missing_required_variable() {
    let tpl = PromptTemplate::new("t1", "Test", "Hello {{name}}!")
        .with_variable(Variable {
            name: "name".into(),
            var_type: VariableType::String,
            default: None,
            description: None,
        });

    let values = HashMap::new();
    let validator = PromptValidator::new();
    let errors = validator.validate_variables(&tpl, &values);
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::MissingVariable { variable, .. } if variable == "name"
    )));
}

#[test]
fn test_validator_duplicate_placeholder() {
    // Template has the same placeholder twice - that's fine, but
    // if we validate we should not flag it as duplicate
    let tpl = PromptTemplate::new("t1", "Test", "{{a}} and {{a}}")
        .with_variable(Variable {
            name: "a".into(),
            var_type: VariableType::String,
            default: None,
            description: None,
        });

    let validator = PromptValidator::new();
    let errors = validator.validate_template(&tpl);
    // Duplicate placeholder should be flagged
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::DuplicateVariable { variable, .. } if variable == "a"
    )));
}

// ─── Version Tests ───────────────────────────────────────────────────────────

#[test]
fn test_version_commit_and_diff() {
    let mut pv = PromptVersion::new("pv1", "Test Version").with_initial_template(
        "## Intro\nHello world\n## Body\nThis is the content",
    );

    assert_eq!(pv.current_version(), 1);

    pv.commit("## Intro\nHello everyone\n## Body\nThis is the content\n## Conclusion\nFinal thoughts");

    assert_eq!(pv.current_version(), 2);

    let diff = pv.diff(1, 2).unwrap();
    assert!(diff.template_changed);
    assert!(diff.change_count() > 0);

    // Intro should be changed
    assert!(matches!(
        diff.section_changes.get("Intro"),
        Some(SectionChange::Changed { .. })
    ));

    // Conclusion should be added
    assert!(matches!(
        diff.section_changes.get("Conclusion"),
        Some(SectionChange::Added { .. })
    ));
}

#[test]
fn test_version_rollback() {
    let mut pv = PromptVersion::new("pv1", "Test").with_initial_template("Version 1");
    pv.commit("Version 2");
    pv.commit("Version 3");

    assert_eq!(pv.current_version(), 3);
    pv.rollback(1).unwrap();
    assert_eq!(pv.current_version(), 1);
    assert_eq!(pv.current_template(), Some("Version 1"));
}

#[test]
fn test_version_diff_empty() {
    let pv = PromptVersion::new("pv1", "Test").with_initial_template("## A\nHello");
    // Diff same version against itself
    let diff = pv.diff(1, 1).unwrap();
    assert!(diff.is_empty());
    assert!(!diff.template_changed);
}

#[test]
fn test_version_list() {
    let mut pv = PromptVersion::new("pv1", "Test").with_initial_template("V1");
    pv.commit("V2");
    pv.commit("V3");
    assert_eq!(pv.list_versions(), vec![1, 2, 3]);
}

#[test]
fn test_version_diff_nonexistent() {
    let pv = PromptVersion::new("pv1", "Test").with_initial_template("V1");
    assert!(pv.diff(1, 99).is_err());
    assert!(pv.diff(99, 1).is_err());
}

// ─── Compiler Tests ──────────────────────────────────────────────────────────

#[test]
fn test_compiler_basic() {
    let t1 = PromptTemplate::new("t1", "System", "You are {{role}}.")
        .with_variable(Variable {
            name: "role".into(),
            var_type: VariableType::String,
            default: None,
            description: None,
        });

    let result = PromptCompiler::new()
        .add_template(t1)
        .set_variable("role", "a helpful AI assistant")
        .compile()
        .unwrap();

    assert_eq!(result, "You are a helpful AI assistant.");
}

#[test]
fn test_compiler_multiple_templates() {
    let t1 = PromptTemplate::new("t1", "System", "You are {{role}}.");
    let t2 = PromptTemplate::new("t2", "User", "Please help me with {{topic}}.");

    let result = PromptCompiler::new()
        .add_template(t1)
        .add_template(t2)
        .set_variable("role", "an expert")
        .set_variable("topic", "Rust")
        .compile()
        .unwrap();

    assert!(result.contains("You are an expert."));
    assert!(result.contains("Please help me with Rust."));
}

#[test]
fn test_compiler_empty() {
    let result = PromptCompiler::new().compile();
    assert!(result.is_err());
}

#[test]
fn test_compiler_strict_validation() {
    let tpl = PromptTemplate::new("t1", "Test", "Hello {{name}}!")
        .with_variable(Variable {
            name: "name".into(),
            var_type: VariableType::String,
            default: None,
            description: None,
        });

    let result = PromptCompiler::new()
        .add_template(tpl)
        .strict()
        .compile();

    assert!(result.is_err());
}

#[test]
fn test_compiler_with_token_budget() {
    let tpl = PromptTemplate::new("t1", "Test", "Word ".repeat(500));

    let result = PromptCompiler::new()
        .add_template(tpl)
        .with_token_budget(TokenBudget::new(10))
        .compile()
        .unwrap();

    // Should be truncated
    assert!(result.len() < 2500);
}

#[test]
fn test_compiler_pipelined() {
    let t1 = PromptTemplate::new("t1", "System", "You are {{role}}.");
    let t2 = PromptTemplate::new("t2", "Task", "Do {{task}}.");

    let result = PromptCompiler::new()
        .add_template(t1)
        .add_template(t2)
        .set_variable("role", "assistant")
        .set_variable("task", "coding")
        .compile_pipelined()
        .unwrap();

    assert!(result.contains("assistant"));
    assert!(result.contains("coding"));
}

#[test]
fn test_compiler_chained() {
    let t1 = PromptTemplate::new("t1", "Step1", "Generate a {{topic}} summary");
    let t2 = PromptTemplate::new("t2", "Step2", "Now expand: {{previous_output}}");

    let result = PromptCompiler::new()
        .add_template(t1)
        .add_template(t2)
        .set_variable("topic", "Rust")
        .compile_chained()
        .unwrap();

    assert!(result.contains("Generate a Rust summary"));
}
