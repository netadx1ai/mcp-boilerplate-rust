//! Prompt types and utilities for MCP server
//!
//! Note: Prompts are now implemented directly in McpServer using the #[prompt] macro
//! from rmcp SDK. This module is kept for backward compatibility and reference types.
//!
//! See src/mcp/stdio_server.rs for the actual prompt implementations:
//! - code_review: Generate code review prompts
//! - explain_code: Generate code explanation prompts  
//! - debug_help: Generate debugging assistance prompts

use serde::{Deserialize, Serialize};

/// Prompt template metadata (for reference/documentation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

/// Prompt argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Get available prompt templates (for documentation purposes)
pub fn get_available_prompts() -> Vec<PromptTemplate> {
    vec![
        PromptTemplate {
            name: "code_review".to_string(),
            description: "Generate a code review prompt for analyzing code quality".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "language".to_string(),
                    description: "Programming language (e.g., rust, python, javascript)"
                        .to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "focus".to_string(),
                    description: "Review focus area (e.g., security, performance, style)"
                        .to_string(),
                    required: false,
                },
            ],
        },
        PromptTemplate {
            name: "explain_code".to_string(),
            description: "Generate a prompt to explain code functionality".to_string(),
            arguments: vec![PromptArgument {
                name: "complexity".to_string(),
                description: "Explanation level (beginner, intermediate, advanced)".to_string(),
                required: false,
            }],
        },
        PromptTemplate {
            name: "debug_help".to_string(),
            description: "Generate a debugging assistance prompt".to_string(),
            arguments: vec![PromptArgument {
                name: "error_type".to_string(),
                description: "Type of error (compile, runtime, logic)".to_string(),
                required: false,
            }],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_available_prompts() {
        let prompts = get_available_prompts();
        assert_eq!(prompts.len(), 3);
        assert_eq!(prompts[0].name, "code_review");
        assert_eq!(prompts[1].name, "explain_code");
        assert_eq!(prompts[2].name, "debug_help");
    }

    #[test]
    fn test_code_review_has_required_args() {
        let prompts = get_available_prompts();
        let code_review = &prompts[0];
        let required_args: Vec<_> = code_review
            .arguments
            .iter()
            .filter(|a| a.required)
            .collect();
        assert_eq!(required_args.len(), 1);
        assert_eq!(required_args[0].name, "language");
    }
}