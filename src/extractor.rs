//! Pincher Multi-Variable Extraction Engine
//! 
//! Pre-compiled regex-based parameter extraction for lightning-fast edge execution.
//! Eliminates the need for runtime LLM calls by pushing variable extraction into
//! the compilation step.
//! 
//! Usage:
//!   let extractor = VariableExtractor::new(vec![
//!       r"sync to branch (?P<target_branch>[a-zA-Z0-9_-]+) with message (?P<commit_message>.*)",
//!       r"push to (?P<target_branch>[a-zA-Z0-9_-]+)",
//!   ])?;
//!   let params = extractor.extract_parameters("sync to branch production with message hotfix");
//!   assert_eq!(params.get("target_branch").unwrap(), "production");

use regex::Regex;
use std::collections::HashMap;

/// Pre-compiled variable extractor using named capture groups
pub struct VariableExtractor {
    compiled_patterns: Vec<Regex>,
}

impl VariableExtractor {
    /// Create a new extractor from regex pattern strings
    pub fn new(patterns: Vec<String>) -> Result<Self, String> {
        let compiled_patterns: Vec<Regex> = patterns
            .iter()
            .map(|p| Regex::new(p).map_err(|e| format!("Invalid regex '{}': {}", p, e)))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(VariableExtractor { compiled_patterns })
    }

    /// Extract named capture groups from input text (runs in <1ms)
    /// 
    /// Returns the first matching pattern's extracted variables.
    /// Falls through all patterns until one matches.
    pub fn extract_parameters(&self, input_text: &str) -> HashMap<String, String> {
        let mut extracted = HashMap::new();

        for regex in &self.compiled_patterns {
            if let Some(captures) = regex.captures(input_text) {
                // Collect all named capture groups
                for name in regex.capture_names().flatten() {
                    if let Some(matched) = captures.name(name) {
                        extracted.insert(name.to_string(), matched.as_str().to_string());
                    }
                }
                if !extracted.is_empty() {
                    break; // First match wins
                }
            }
        }

        extracted
    }

    /// Returns true if the input matches any compiled pattern
    pub fn matches(&self, input_text: &str) -> bool {
        self.compiled_patterns.iter().any(|r| r.is_match(input_text))
    }

    /// Get the number of compiled patterns
    pub fn pattern_count(&self) -> usize {
        self.compiled_patterns.len()
    }

    /// Validate that all variables in a schema are extractable from seed phrases
    pub fn validate_schema_coverage(
        &self,
        schema_vars: &[String],
        seed_phrases: &[String],
    ) -> HashMap<String, Vec<String>> {
        let mut uncovered: HashMap<String, Vec<String>> = HashMap::new();

        for var in schema_vars {
            for seed in seed_phrases {
                let params = self.extract_parameters(seed);
                if !params.contains_key(var) {
                    uncovered
                        .entry(var.clone())
                        .or_default()
                        .push(seed.clone());
                }
            }
        }

        uncovered
    }
}

/// Fallback tiny local model for when regex patterns don't match the user input
/// Uses basic keyword extraction as a lightweight fallback
pub struct KeywordFallbackExtractor;

impl KeywordFallbackExtractor {
    /// Extract simple key=value patterns using basic keyword matching
    pub fn extract(&self, input_text: &str, known_keys: &[&str]) -> HashMap<String, String> {
        let mut extracted = HashMap::new();
        let lowercase = input_text.to_lowercase();

        for key in known_keys {
            // Look for patterns like "key = value" or "key: value" or "key value"
            for prefix in &[format!("{}=", key), format!("{}:", key), format!("{} ", key)] {
                if let Some(pos) = lowercase.find(prefix.as_str()) {
                    let start = pos + prefix.len();
                    let remaining = &input_text[start..];
                    // Take until the next known key or end of string
                    let value = remaining
                        .split(|c: char| c == ' ' || c == ',' || c == ';')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if !value.is_empty() {
                        extracted.insert(key.to_string(), value);
                        break;
                    }
                }
            }
        }

        extracted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_variable_extraction() {
        let patterns = vec![
            r"sync to branch (?P<target_branch>[a-zA-Z0-9_-]+) with message (?P<commit_message>.*)".to_string(),
            r"push to (?P<target_branch>[a-zA-Z0-9_-]+)".to_string(),
        ];

        let extractor = VariableExtractor::new(patterns).unwrap();
        let params = extractor.extract_parameters("sync to branch production with message hotfix deploy");

        assert_eq!(params.get("target_branch").unwrap(), "production");
        assert_eq!(params.get("commit_message").unwrap(), "hotfix deploy");
    }

    #[test]
    fn test_second_pattern_fallback() {
        let patterns = vec![
            r"sync to branch (?P<target_branch>[a-zA-Z0-9_-]+) with message (?P<commit_message>.*)".to_string(),
            r"push to (?P<target_branch>[a-zA-Z0-9_-]+)".to_string(),
        ];

        let extractor = VariableExtractor::new(patterns).unwrap();
        let params = extractor.extract_parameters("push to main");

        assert_eq!(params.get("target_branch").unwrap(), "main");
        assert!(params.get("commit_message").is_none()); // Not in this pattern
    }

    #[test]
    fn test_no_match_returns_empty() {
        let patterns = vec![
            r"sync to branch (?P<target_branch>[a-zA-Z0-9_-]+)".to_string(),
        ];

        let extractor = VariableExtractor::new(patterns).unwrap();
        let params = extractor.extract_parameters("hello world");

        assert!(params.is_empty());
    }

    #[test]
    fn test_schema_coverage() {
        let patterns = vec![
            r"sync to branch (?P<target_branch>[a-zA-Z0-9_-]+)".to_string(),
        ];

        let extractor = VariableExtractor::new(patterns).unwrap();
        let schema_vars = vec!["target_branch".to_string(), "commit_message".to_string()];
        let seeds = vec!["sync to branch staging".to_string()];

        let uncovered = extractor.validate_schema_coverage(&schema_vars, &seeds);
        assert!(uncovered.contains_key("commit_message"));
        assert!(!uncovered.contains_key("target_branch"));
    }

    #[test]
    fn test_keyword_fallback() {
        let fallback = KeywordFallbackExtractor;
        let params = fallback.extract("Set target_branch=production and deploy", &["target_branch"]);

        assert_eq!(params.get("target_branch").unwrap(), "production");
    }
}
