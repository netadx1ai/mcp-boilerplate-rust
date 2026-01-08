use rmcp::{
    ErrorData as McpError,
    handler::server::tool::ToolRouter,
    handler::server::wrapper::{Json, Parameters},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Clone)]
pub struct CalculatorTool {
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct CalculateRequest {
    pub operation: String,
    pub a: f64,
    pub b: f64,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct CalculateResponse {
    pub operation: String,
    pub a: f64,
    pub b: f64,
    pub result: f64,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct EvaluateRequest {
    pub expression: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct EvaluateResponse {
    pub expression: String,
    pub result: f64,
    pub timestamp: String,
}

#[tool_router]
impl CalculatorTool {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[allow(dead_code)]
    pub fn router(&self) -> &ToolRouter<Self> {
        &self.tool_router
    }

    #[tool(description = "Perform basic arithmetic operations (add, subtract, multiply, divide)")]
    pub async fn calculate(
        &self,
        params: Parameters<CalculateRequest>,
    ) -> Result<Json<CalculateResponse>, McpError> {
        let req = params.0;
        
        info!("Calculate: {} {} {}", req.a, req.operation, req.b);
        
        let result = match req.operation.to_lowercase().as_str() {
            "add" | "+" => req.a + req.b,
            "subtract" | "-" => req.a - req.b,
            "multiply" | "*" => req.a * req.b,
            "divide" | "/" => {
                if req.b == 0.0 {
                    return Err(McpError::invalid_params(
                        "Division by zero is not allowed".to_string(),
                        None,
                    ));
                }
                req.a / req.b
            }
            "modulo" | "%" => {
                if req.b == 0.0 {
                    return Err(McpError::invalid_params(
                        "Modulo by zero is not allowed".to_string(),
                        None,
                    ));
                }
                req.a % req.b
            }
            "power" | "pow" | "^" => req.a.powf(req.b),
            _ => {
                return Err(McpError::invalid_params(
                    format!(
                        "Unknown operation: '{}'. Supported: add, subtract, multiply, divide, modulo, power",
                        req.operation
                    ),
                    None,
                ));
            }
        };

        if !result.is_finite() {
            return Err(McpError::invalid_params(
                "Result is not a finite number (overflow or invalid operation)".to_string(),
                None,
            ));
        }

        Ok(Json(CalculateResponse {
            operation: req.operation,
            a: req.a,
            b: req.b,
            result,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }

    #[tool(description = "Evaluate a mathematical expression (supports +, -, *, /, parentheses)")]
    pub async fn evaluate(
        &self,
        params: Parameters<EvaluateRequest>,
    ) -> Result<Json<EvaluateResponse>, McpError> {
        let expression = params.0.expression.trim();
        
        if expression.is_empty() {
            return Err(McpError::invalid_params(
                "Expression cannot be empty".to_string(),
                None,
            ));
        }

        if expression.len() > 1000 {
            return Err(McpError::invalid_params(
                "Expression too long (max 1000 characters)".to_string(),
                None,
            ));
        }

        info!("Evaluate: {}", expression);

        let result = evaluate_expression(expression).map_err(|e| {
            McpError::invalid_params(format!("Failed to evaluate expression: {}", e), None)
        })?;

        if !result.is_finite() {
            return Err(McpError::invalid_params(
                "Result is not a finite number".to_string(),
                None,
            ));
        }

        Ok(Json(EvaluateResponse {
            expression: expression.to_string(),
            result,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }
}

impl Default for CalculatorTool {
    fn default() -> Self {
        Self::new()
    }
}

fn evaluate_expression(expr: &str) -> Result<f64, String> {
    let expr = expr.replace(" ", "");
    
    for c in expr.chars() {
        if !c.is_ascii_digit() && !matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '.') {
            return Err(format!("Invalid character in expression: '{}'", c));
        }
    }
    
    parse_expression(&expr, 0).map(|(result, _)| result)
}

fn parse_expression(expr: &str, pos: usize) -> Result<(f64, usize), String> {
    let (mut left, mut pos) = parse_term(expr, pos)?;
    
    while pos < expr.len() {
        let op = expr.chars().nth(pos).unwrap();
        match op {
            '+' | '-' => {
                let (right, new_pos) = parse_term(expr, pos + 1)?;
                if op == '+' {
                    left += right;
                } else {
                    left -= right;
                }
                pos = new_pos;
            }
            ')' => break,
            _ => return Err(format!("Unexpected character at position {}: '{}'", pos, op)),
        }
    }
    
    Ok((left, pos))
}

fn parse_term(expr: &str, pos: usize) -> Result<(f64, usize), String> {
    let (mut left, mut pos) = parse_factor(expr, pos)?;
    
    while pos < expr.len() {
        let op = expr.chars().nth(pos).unwrap();
        match op {
            '*' | '/' => {
                let (right, new_pos) = parse_factor(expr, pos + 1)?;
                if op == '*' {
                    left *= right;
                } else {
                    if right == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                    left /= right;
                }
                pos = new_pos;
            }
            '+' | '-' | ')' => break,
            _ => return Err(format!("Unexpected character at position {}: '{}'", pos, op)),
        }
    }
    
    Ok((left, pos))
}

fn parse_factor(expr: &str, pos: usize) -> Result<(f64, usize), String> {
    if pos >= expr.len() {
        return Err("Unexpected end of expression".to_string());
    }
    
    let ch = expr.chars().nth(pos).unwrap();
    
    if ch == '(' {
        let (result, new_pos) = parse_expression(expr, pos + 1)?;
        if new_pos >= expr.len() || expr.chars().nth(new_pos).unwrap() != ')' {
            return Err("Missing closing parenthesis".to_string());
        }
        Ok((result, new_pos + 1))
    } else if ch == '-' || ch == '+' {
        let (result, new_pos) = parse_factor(expr, pos + 1)?;
        if ch == '-' {
            Ok((-result, new_pos))
        } else {
            Ok((result, new_pos))
        }
    } else if ch.is_ascii_digit() || ch == '.' {
        parse_number(expr, pos)
    } else {
        Err(format!("Unexpected character at position {}: '{}'", pos, ch))
    }
}

fn parse_number(expr: &str, pos: usize) -> Result<(f64, usize), String> {
    let mut end = pos;
    let mut has_dot = false;
    
    while end < expr.len() {
        let ch = expr.chars().nth(end).unwrap();
        if ch.is_ascii_digit() {
            end += 1;
        } else if ch == '.' && !has_dot {
            has_dot = true;
            end += 1;
        } else {
            break;
        }
    }
    
    if end == pos {
        return Err(format!("Expected number at position {}", pos));
    }
    
    let num_str = &expr[pos..end];
    let num = num_str.parse::<f64>()
        .map_err(|_| format!("Invalid number: '{}'", num_str))?;
    
    Ok((num, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_simple() {
        assert_eq!(evaluate_expression("2+2").unwrap(), 4.0);
        assert_eq!(evaluate_expression("10-5").unwrap(), 5.0);
        assert_eq!(evaluate_expression("3*4").unwrap(), 12.0);
        assert_eq!(evaluate_expression("15/3").unwrap(), 5.0);
    }

    #[test]
    fn test_evaluate_precedence() {
        assert_eq!(evaluate_expression("2+3*4").unwrap(), 14.0);
        assert_eq!(evaluate_expression("10-2*3").unwrap(), 4.0);
    }

    #[test]
    fn test_evaluate_parentheses() {
        assert_eq!(evaluate_expression("(2+3)*4").unwrap(), 20.0);
        assert_eq!(evaluate_expression("2*(3+4)").unwrap(), 14.0);
    }

    #[test]
    fn test_evaluate_negative() {
        assert_eq!(evaluate_expression("-5+3").unwrap(), -2.0);
        assert_eq!(evaluate_expression("10+-5").unwrap(), 5.0);
    }

    #[test]
    fn test_evaluate_decimal() {
        assert_eq!(evaluate_expression("2.5+1.5").unwrap(), 4.0);
        assert_eq!(evaluate_expression("10.5/2").unwrap(), 5.25);
    }

    #[test]
    fn test_evaluate_complex() {
        assert_eq!(evaluate_expression("(2+3)*(4-1)").unwrap(), 15.0);
        assert_eq!(evaluate_expression("2*(3+4*5)").unwrap(), 46.0);
    }

    #[test]
    fn test_evaluate_division_by_zero() {
        assert!(evaluate_expression("5/0").is_err());
    }

    #[test]
    fn test_evaluate_invalid() {
        assert!(evaluate_expression("2+").is_err());
        assert!(evaluate_expression("(2+3").is_err());
        assert!(evaluate_expression("2**3").is_err());
        assert!(evaluate_expression("abc").is_err());
    }
}