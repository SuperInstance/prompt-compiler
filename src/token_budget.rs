/// Error from token budget operations.
#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("Token limit exceeded: estimated {estimated} tokens, max is {max}")]
    Exceeded { estimated: usize, max: usize },
    #[error("Token estimation failed for text: {0}")]
    EstimationFailed(String),
}

/// Token budget estimator and enforcer.
///
/// Uses a simple heuristic: ~4 characters per token (rough GPT-style estimate).
/// For production use, integrate a real tokenizer.
#[derive(Debug, Clone)]
pub struct TokenBudget {
    max_tokens: usize,
    chars_per_token: f64,
}

impl TokenBudget {
    /// Create a new token budget with the given maximum.
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            chars_per_token: 4.0,
        }
    }

    /// Set a custom characters-per-token ratio.
    pub fn with_chars_per_token(mut self, ratio: f64) -> Self {
        self.chars_per_token = ratio;
        self
    }

    /// Get the maximum token count.
    pub fn max_tokens(&self) -> usize {
        self.max_tokens
    }

    /// Estimate the number of tokens in a text string.
    pub fn estimate_tokens(&self, text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }
        // Simple heuristic: split by whitespace and estimate per-word
        let char_count = text.len();
        let estimate = (char_count as f64 / self.chars_per_token).ceil() as usize;
        // Minimum: whitespace-based word count is also a decent proxy
        let word_count = text.split_whitespace().count();
        // Take the larger of the two estimates for safety
        estimate.max(word_count)
    }

    /// Check if text is within budget.
    pub fn is_within_budget(&self, text: &str) -> bool {
        self.estimate_tokens(text) <= self.max_tokens
    }

    /// Validate that text is within budget, returning an error if not.
    pub fn validate(&self, text: &str) -> Result<usize, TokenError> {
        let count = self.estimate_tokens(text);
        if count > self.max_tokens {
            Err(TokenError::Exceeded {
                estimated: count,
                max: self.max_tokens,
            })
        } else {
            Ok(count)
        }
    }

    /// Truncate text to fit within the token budget.
    /// Tries to break at sentence boundaries or word boundaries.
    pub fn truncate_to_budget(&self, text: &str) -> String {
        if self.is_within_budget(text) {
            return text.to_string();
        }

        // Target max characters based on token ratio
        let max_chars = (self.max_tokens as f64 * self.chars_per_token) as usize;

        if text.len() <= max_chars {
            return text.to_string();
        }

        // Find a good break point near max_chars
        let slice = &text[..max_chars.min(text.len())];

        // Try sentence boundary first
        if let Some(pos) = slice.rfind(". ") {
            return format!("{}.", &text[..pos]);
        }
        if let Some(pos) = slice.rfind("。") {
            return text[..pos + "。".len()].to_string();
        }

        // Try word boundary
        if let Some(pos) = slice.rfind(' ') {
            return text[..pos].to_string();
        }

        // Hard cut
        text[..max_chars].to_string()
    }

    /// Calculate how many tokens remain in the budget after the given text.
    pub fn remaining(&self, text: &str) -> usize {
        self.max_tokens.saturating_sub(self.estimate_tokens(text))
    }
}
