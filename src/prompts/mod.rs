use rmcp::model::{GetPromptResult, Prompt, PromptMessage, PromptMessageRole};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

#[derive(Clone)]
pub struct PromptRegistry {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptRegistry {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        templates.insert(
            "code_review".to_string(),
            PromptTemplate {
                name: "code_review".to_string(),
                description: "Generate a code review prompt for analyzing code quality".to_string(),
                arguments: vec![
                    PromptArgument {
                        name: "language".to_string(),
                        description: "Programming language (e.g., rust, python, javascript)".to_string(),
                        required: true,
                    },
                    PromptArgument {
                        name: "focus".to_string(),
                        description: "Review focus area (e.g., security, performance, style)".to_string(),
                        required: false,
                    },
                ],
            },
        );

        templates.insert(
            "explain_code".to_string(),
            PromptTemplate {
                name: "explain_code".to_string(),
                description: "Generate a prompt to explain code functionality".to_string(),
                arguments: vec![
                    PromptArgument {
                        name: "complexity".to_string(),
                        description: "Explanation level (beginner, intermediate, advanced)".to_string(),
                        required: false,
                    },
                ],
            },
        );

        templates.insert(
            "debug_help".to_string(),
            PromptTemplate {
                name: "debug_help".to_string(),
                description: "Generate a debugging assistance prompt".to_string(),
                arguments: vec![
                    PromptArgument {
                        name: "error_type".to_string(),
                        description: "Type of error (compile, runtime, logic)".to_string(),
                        required: false,
                    },
                ],
            },
        );

        Self { templates }
    }

    pub fn list_prompts(&self) -> Vec<Prompt> {
        self.templates
            .values()
            .map(|template| Prompt {
                name: template.name.clone(),
                title: None,
                description: Some(template.description.clone()),
                arguments: Some(
                    template
                        .arguments
                        .iter()
                        .map(|arg| rmcp::model::PromptArgument {
                            name: arg.name.clone(),
                            title: None,
                            description: Some(arg.description.clone()),
                            required: Some(arg.required),
                        })
                        .collect(),
                ),
                icons: None,
                meta: None,
            })
            .collect()
    }

    pub fn get_prompt(
        &self,
        name: &str,
        arguments: &HashMap<String, String>,
    ) -> Result<GetPromptResult, String> {
        let template = self
            .templates
            .get(name)
            .ok_or_else(|| format!("Prompt '{}' not found", name))?;

        for arg in &template.arguments {
            if arg.required && !arguments.contains_key(&arg.name) {
                return Err(format!("Required argument '{}' is missing", arg.name));
            }
        }

        let messages = self.generate_prompt_messages(name, arguments)?;

        Ok(GetPromptResult {
            description: Some(template.description.clone()),
            messages,
        })
    }

    fn generate_prompt_messages(
        &self,
        name: &str,
        arguments: &HashMap<String, String>,
    ) -> Result<Vec<PromptMessage>, String> {
        match name {
            "code_review" => {
                let language = arguments
                    .get("language")
                    .ok_or("language argument required")?;
                let focus = arguments.get("focus").map(|s| s.as_str()).unwrap_or("general");

                Ok(vec![PromptMessage::new_text(
                    PromptMessageRole::User,
                    format!(
                        "Please review the following {} code with a focus on {}. \
                        Provide detailed feedback on:\n\
                        1. Code quality and best practices\n\
                        2. Potential bugs or issues\n\
                        3. Performance considerations\n\
                        4. Security vulnerabilities\n\
                        5. Suggestions for improvement",
                        language, focus
                    ),
                )])
            }
            "explain_code" => {
                let complexity = arguments
                    .get("complexity")
                    .map(|s| s.as_str())
                    .unwrap_or("intermediate");

                let level_desc = match complexity {
                    "beginner" => "in simple terms suitable for beginners",
                    "advanced" => "with technical depth for experienced developers",
                    _ => "at an intermediate level",
                };

                Ok(vec![PromptMessage::new_text(
                    PromptMessageRole::User,
                    format!(
                        "Please explain the following code {}. Include:\n\
                        1. What the code does\n\
                        2. How it works step by step\n\
                        3. Key concepts and patterns used\n\
                        4. Any important considerations",
                        level_desc
                    ),
                )])
            }
            "debug_help" => {
                let error_type = arguments
                    .get("error_type")
                    .map(|s| s.as_str())
                    .unwrap_or("general");

                Ok(vec![PromptMessage::new_text(
                    PromptMessageRole::User,
                    format!(
                        "Help me debug this {} error. Please:\n\
                        1. Analyze the error message and code\n\
                        2. Identify the root cause\n\
                        3. Suggest specific fixes\n\
                        4. Explain why the error occurred\n\
                        5. Recommend preventive measures",
                        error_type
                    ),
                )])
            }
            _ => Err(format!("Unknown prompt template: {}", name)),
        }
    }
}

impl Default for PromptRegistry {
    fn default() -> Self {
        Self::new()
    }
}