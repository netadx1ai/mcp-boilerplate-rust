//! TextGen Tool -- AI text generation via MCP V5 proxy
//!
//! DTV BE does NOT call Gemini/OpenAI directly. All AI generation proxies through MCP V5:
//!   FE -> DTV BE (textgen tool) -> MCP V5 (api_v5.ainext.vn/tools/text_generation) -> Gemini/OpenAI
//!
//! V5 API contract:
//! - Action: "generate_text"
//! - Prompt format: flat `prompt` + `system_prompt` (not messages[] array)
//! - Pass `response_format` through to V5 as-is (including json_schema type)
//! - Always `bypassConsume: true` (DTV manages credits in PostgreSQL via dtv_ tables)

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::sync::OnceLock;
use tracing::{debug, error, info, warn};

// ---------------------------------------------------------------------------
// HTTP client singleton
// ---------------------------------------------------------------------------

static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

fn get_http_client() -> &'static Client {
    HTTP_CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to build HTTP client")
    })
}

// ---------------------------------------------------------------------------
// Config helpers
// ---------------------------------------------------------------------------

fn v5_api_url() -> String {
    env::var("V5_API_URL").unwrap_or_else(|_| "http://api_v5.ainext.vn".to_string())
}

fn v5_api_key() -> Option<String> {
    env::var("V5_API_KEY").ok()
}

fn jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| "aivaAPI".to_string())
}

fn postgrest_url() -> String {
    env::var("POSTGREST_URL").unwrap_or_else(|_| "http://localhost:3001".to_string())
}

fn db_prefix() -> String {
    env::var("DB_TABLE_PREFIX").unwrap_or_else(|_| "dtv_".to_string())
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

/// Incoming request from FE (via mcpClient.callTool('textgen', {...}))
#[derive(Debug, Deserialize)]
pub struct TextGenInput {
    /// Action -- currently only "generate"
    #[allow(dead_code)]
    pub action: Option<String>,
    /// User JWT token (auto-injected by mcpClient.callTool)
    pub token: Option<String>,
    /// Text prompt (required)
    pub prompt: Option<String>,
    /// Model code passed through to V5 (default: gemini-2.5-pro)
    pub model_code: Option<String>,
    /// System prompt
    pub system_prompt: Option<String>,
    /// Max tokens
    pub max_tokens: Option<u32>,
    /// Temperature
    pub temperature: Option<f32>,
    /// Simple JSON mode flag
    pub json_mode: Option<bool>,
    /// Structured response format (passed through to V5 as-is)
    pub response_format: Option<Value>,
    /// Tool ID for credit deduction (e.g. "career_assessment", "horoscope_daily")
    #[serde(alias = "toolId")]
    pub tool_id: Option<String>,
    /// Vision attachments
    pub attachments: Option<Vec<Attachment>>,
    /// Whether to save result to dtv_user_results
    pub save_result: Option<SaveResultOpts>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Attachment {
    #[serde(rename = "type")]
    pub att_type: String,
    pub url: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveResultOpts {
    #[serde(alias = "toolId")]
    pub tool_id: Option<String>,
    #[serde(alias = "resultSummary")]
    pub result_summary: Option<String>,
}

/// Outgoing request to MCP V5
#[derive(Debug, Serialize)]
struct V5Request {
    action: String,
    model_code: String,
    #[serde(rename = "userId")]
    user_id: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    json_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<Value>,
    #[serde(rename = "bypassConsume")]
    bypass_consume: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments: Option<Vec<Attachment>>,
}

/// Standard tool response
#[derive(Debug, Serialize)]
pub struct TextGenResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub metadata: TextGenMetadata,
}

#[derive(Debug, Serialize)]
pub struct TextGenMetadata {
    #[serde(rename = "executionTime")]
    pub execution_time: u128,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "creditsUsed")]
    pub credits_used: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "remainingCredits")]
    pub remaining_credits: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "resultId")]
    pub result_id: Option<String>,
}

// ---------------------------------------------------------------------------
// JWT decode (minimal -- just extract user id, same as auth module)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct JwtClaims {
    #[serde(alias = "userId", alias = "user_id")]
    user_id: Option<String>,
    sub: Option<String>,
}

fn decode_token(token: &str) -> Result<String, String> {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    let secret = jwt_secret();
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false;
    validation.required_spec_claims.clear();

    let token_data = decode::<JwtClaims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
        .map_err(|e| format!("Invalid token: {e}"))?;

    token_data
        .claims
        .user_id
        .or(token_data.claims.sub)
        .ok_or_else(|| "Token missing user_id/sub claim".to_string())
}

// ---------------------------------------------------------------------------
// Credit check & deduct via PostgREST
// ---------------------------------------------------------------------------

struct CreditResult {
    success: bool,
    error: Option<String>,
    credits_used: i32,
    remaining: Option<i64>,
}

async fn check_and_deduct_credits(user_id: &str, tool_id: &str) -> CreditResult {
    let client = get_http_client();
    let base = postgrest_url();
    let prefix = db_prefix();

    // 1. Get tool settings (DTV schema: tool_id, cost, free_daily_limit, is_active)
    let tool_settings_url = format!(
        "{}/{prefix}tool_settings?tool_id=eq.{tool_id}&select=cost,free_daily_limit,is_active",
        base
    );

    let settings_resp = match client.get(&tool_settings_url).send().await {
        Ok(r) => r,
        Err(e) => {
            error!("[textgen] Failed to fetch tool_settings: {e}");
            return CreditResult {
                success: false,
                error: Some(format!("Failed to fetch tool settings: {e}")),
                credits_used: 0,
                remaining: None,
            };
        }
    };

    let settings: Vec<Value> = match settings_resp.json().await {
        Ok(v) => v,
        Err(e) => {
            error!("[textgen] Failed to parse tool_settings: {e}");
            return CreditResult {
                success: false,
                error: Some(format!("Invalid tool settings response: {e}")),
                credits_used: 0,
                remaining: None,
            };
        }
    };

    if settings.is_empty() {
        warn!("[textgen] Tool settings not found for: {tool_id}");
        // If no tool_settings entry, allow free (no credit cost)
        return CreditResult {
            success: true,
            error: None,
            credits_used: 0,
            remaining: None,
        };
    }

    let setting = &settings[0];
    let credit_cost = setting["cost"].as_i64().unwrap_or(0) as i32;
    let free_per_day = setting["free_daily_limit"].as_i64().unwrap_or(0) as i32;
    let is_active = setting["is_active"].as_bool().unwrap_or(true);

    if !is_active {
        return CreditResult {
            success: false,
            error: Some("Công cụ đang tạm ngưng".to_string()),
            credits_used: 0,
            remaining: None,
        };
    }

    // If tool is free (cost 0), just track usage
    if credit_cost == 0 {
        let _ = upsert_usage(user_id, tool_id, 0).await;
        return CreditResult {
            success: true,
            error: None,
            credits_used: 0,
            remaining: None,
        };
    }

    // 2. Check free uses today
    // DTV schema: dtv_credit_usage(user_id, tool_id, date, count)
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let usage_url = format!(
        "{}/{prefix}credit_usage?user_id=eq.{user_id}&tool_id=eq.{tool_id}&date=eq.{today}&select=count",
        base
    );

    let usage_count = match client.get(&usage_url).send().await {
        Ok(resp) => {
            let rows: Vec<Value> = resp.json().await.unwrap_or_default();
            rows.first()
                .and_then(|r| r["count"].as_i64())
                .unwrap_or(0) as i32
        }
        Err(_) => 0,
    };

    if free_per_day > 0 && usage_count < free_per_day {
        // Use free credit
        let _ = upsert_usage(user_id, tool_id, 0).await;
        debug!("[textgen] Free usage {}/{} for user {} tool {}", usage_count + 1, free_per_day, user_id, tool_id);
        return CreditResult {
            success: true,
            error: None,
            credits_used: 0,
            remaining: None,
        };
    }

    // 3. Check wallet balance
    // DTV schema: dtv_credit_wallets(user_id, bonus_credits, referral_credits, paid_credits)
    let wallet_url = format!(
        "{}/{prefix}credit_wallets?user_id=eq.{user_id}&select=id,bonus_credits,referral_credits,paid_credits",
        base
    );

    let wallet_resp = match client.get(&wallet_url).send().await {
        Ok(r) => r,
        Err(e) => {
            error!("[textgen] Failed to fetch wallet: {e}");
            return CreditResult {
                success: false,
                error: Some("Không thể kiểm tra ví tín dụng".to_string()),
                credits_used: 0,
                remaining: None,
            };
        }
    };

    let wallets: Vec<Value> = wallet_resp.json().await.unwrap_or_default();
    if wallets.is_empty() {
        return CreditResult {
            success: false,
            error: Some("Không tìm thấy ví tín dụng. Vui lòng đăng nhập lại.".to_string()),
            credits_used: 0,
            remaining: None,
        };
    }

    let wallet = &wallets[0];
    let bonus = wallet["bonus_credits"].as_i64().unwrap_or(0);
    let referral = wallet["referral_credits"].as_i64().unwrap_or(0);
    let paid = wallet["paid_credits"].as_i64().unwrap_or(0);
    let total = bonus + referral + paid;
    let cost = credit_cost as i64;

    if total < cost {
        return CreditResult {
            success: false,
            error: Some(format!("Không đủ tín dụng. Cần {} tín dụng, còn {} tín dụng.", cost, total)),
            credits_used: 0,
            remaining: Some(total),
        };
    }

    // 4. Deduct credits (priority: bonus -> referral -> paid)
    let mut remaining_deduct = cost;

    let bonus_deduct = std::cmp::min(bonus, remaining_deduct);
    remaining_deduct -= bonus_deduct;

    let referral_deduct = std::cmp::min(referral, remaining_deduct);
    remaining_deduct -= referral_deduct;

    let paid_deduct = remaining_deduct;

    let wallet_id = wallet["id"].as_str().unwrap_or("");
    let new_bonus = bonus - bonus_deduct;
    let new_referral = referral - referral_deduct;
    let new_paid = paid - paid_deduct;
    let new_total = new_bonus + new_referral + new_paid;

    let update_url = format!(
        "{}/{prefix}credit_wallets?id=eq.{wallet_id}",
        base
    );

    let update_body = json!({
        "bonus_credits": new_bonus,
        "referral_credits": new_referral,
        "paid_credits": new_paid,
        "updated_at": chrono::Utc::now().to_rfc3339()
    });

    if let Err(e) = client
        .patch(&update_url)
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&update_body)
        .send()
        .await
    {
        error!("[textgen] Failed to update wallet: {e}");
        return CreditResult {
            success: false,
            error: Some("Lỗi trừ tín dụng".to_string()),
            credits_used: 0,
            remaining: Some(total),
        };
    }

    // 5. Record usage + transaction
    let _ = upsert_usage(user_id, tool_id, credit_cost).await;
    let _ = record_transaction(user_id, tool_id, credit_cost).await;

    info!("[textgen] Deducted {} credits for user {} tool {} (remaining: {})", credit_cost, user_id, tool_id, new_total);

    CreditResult {
        success: true,
        error: None,
        credits_used: credit_cost,
        remaining: Some(new_total),
    }
}

/// Upsert daily usage counter
/// DTV schema: dtv_credit_usage(id, user_id, tool_id, date, count, created_at)
/// Unique index on (user_id, tool_id, date)
async fn upsert_usage(user_id: &str, tool_id: &str, _credits_spent: i32) {
    let client = get_http_client();
    let base = postgrest_url();
    let prefix = db_prefix();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Check if usage row exists
    let check_url = format!(
        "{}/{prefix}credit_usage?user_id=eq.{user_id}&tool_id=eq.{tool_id}&date=eq.{today}&select=id,count",
        base
    );

    let existing = match client.get(&check_url).send().await {
        Ok(r) => r.json::<Vec<Value>>().await.unwrap_or_default(),
        Err(_) => vec![],
    };

    if let Some(row) = existing.first() {
        // Update existing row -- increment count
        let current_count = row["count"].as_i64().unwrap_or(0);
        let row_id = row["id"].as_str().unwrap_or("");

        let update_url = format!(
            "{}/{prefix}credit_usage?id=eq.{row_id}",
            base
        );

        let update_body = json!({
            "count": current_count + 1
        });

        let _ = client
            .patch(&update_url)
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(&update_body)
            .send()
            .await;
    } else {
        // Insert new row
        let insert_url = format!("{}/{prefix}credit_usage", base);
        let insert_body = json!({
            "user_id": user_id,
            "tool_id": tool_id,
            "date": today,
            "count": 1,
            "created_at": now
        });

        let _ = client
            .post(&insert_url)
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(&insert_body)
            .send()
            .await;
    }
}

/// Record credit transaction
/// DTV schema: dtv_credit_transactions(id, user_id, tool_id, amount, type, description, created_at)
async fn record_transaction(user_id: &str, tool_id: &str, amount: i32) {
    let client = get_http_client();
    let base = postgrest_url();
    let prefix = db_prefix();
    let now = chrono::Utc::now().to_rfc3339();

    let url = format!("{}/{prefix}credit_transactions", base);
    let body = json!({
        "user_id": user_id,
        "type": "usage",
        "amount": -(amount as i64),
        "description": format!("AI generation: {}", tool_id),
        "tool_id": tool_id,
        "created_at": now
    });

    let _ = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&body)
        .send()
        .await;
}

// ---------------------------------------------------------------------------
// Save result to dtv_user_results (optional)
// ---------------------------------------------------------------------------

async fn save_result(user_id: &str, tool_id: &str, data: &Value, summary: Option<&str>) -> Option<String> {
    let client = get_http_client();
    let base = postgrest_url();
    let prefix = db_prefix();
    let now = chrono::Utc::now().to_rfc3339();
    let share_slug = format!("dtv_{}", chrono::Utc::now().timestamp_millis());

    let url = format!("{}/{prefix}user_results", base);
    let body = json!({
        "user_id": user_id,
        "tool_id": tool_id,
        "result_data": data,
        "result_summary": summary,
        "share_slug": share_slug,
        "created_at": now
    });

    match client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Prefer", "return=representation")
        .json(&body)
        .send()
        .await
    {
        Ok(resp) => {
            let rows: Vec<Value> = resp.json().await.unwrap_or_default();
            rows.first()
                .and_then(|r| r["id"].as_str().map(|s| s.to_string()))
                .or_else(|| rows.first().and_then(|r| r["id"].as_i64().map(|n| n.to_string())))
        }
        Err(e) => {
            error!("[textgen] Failed to save result: {e}");
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Call MCP V5 text generation API
// ---------------------------------------------------------------------------

async fn call_v5(request: &V5Request) -> Result<Value, String> {
    let client = get_http_client();
    let url = format!("{}/tools/text_generation", v5_api_url());
    let api_key = v5_api_key().ok_or("V5_API_KEY not configured")?;

    debug!("[textgen] Calling V5: {} model={}", url, request.model_code);

    let resp = client
        .post(&url)
        .header("X-API-Key", &api_key)
        .header("Content-Type", "application/json")
        .json(request)
        .send()
        .await
        .map_err(|e| format!("V5 request failed: {e}"))?;

    let status = resp.status();
    let body: Value = resp
        .json()
        .await
        .map_err(|e| format!("V5 response parse failed: {e}"))?;

    if !status.is_success() {
        let err_msg = body["error"]
            .as_str()
            .or_else(|| body["message"].as_str())
            .unwrap_or("Unknown V5 error");
        error!("[textgen] V5 returned {}: {}", status, err_msg);
        return Err(format!("V5 error ({}): {}", status, err_msg));
    }

    Ok(body)
}

// ---------------------------------------------------------------------------
// Main execute function
// ---------------------------------------------------------------------------

pub async fn execute(args: Value) -> Value {
    let start = std::time::Instant::now();
    let now_str = chrono::Utc::now().to_rfc3339();

    // Parse input
    let input: TextGenInput = match serde_json::from_value(args) {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "success": false,
                "error": format!("Invalid textgen request: {e}"),
                "metadata": { "executionTime": start.elapsed().as_millis(), "timestamp": now_str }
            });
        }
    };

    // Validate required fields
    let prompt = match &input.prompt {
        Some(p) if !p.trim().is_empty() => p.clone(),
        _ => {
            return json!({
                "success": false,
                "error": "prompt is required",
                "metadata": { "executionTime": start.elapsed().as_millis(), "timestamp": now_str }
            });
        }
    };

    let token = match &input.token {
        Some(t) if !t.is_empty() => t.clone(),
        _ => {
            return json!({
                "success": false,
                "error": "token is required",
                "metadata": { "executionTime": start.elapsed().as_millis(), "timestamp": now_str }
            });
        }
    };

    // 1. Verify JWT
    let user_id = match decode_token(&token) {
        Ok(id) => id,
        Err(e) => {
            return json!({
                "success": false,
                "error": e,
                "metadata": { "executionTime": start.elapsed().as_millis(), "timestamp": now_str }
            });
        }
    };

    // 2. Credit check & deduct (if toolId provided)
    let mut credits_used: Option<i32> = None;
    let mut remaining_credits: Option<i64> = None;

    if let Some(ref tool_id) = input.tool_id {
        if !tool_id.is_empty() {
            let credit_result = check_and_deduct_credits(&user_id, tool_id).await;
            if !credit_result.success {
                return json!({
                    "success": false,
                    "error": credit_result.error.unwrap_or("Credit check failed".to_string()),
                    "metadata": {
                        "executionTime": start.elapsed().as_millis(),
                        "timestamp": now_str,
                        "remainingCredits": credit_result.remaining
                    }
                });
            }
            credits_used = Some(credit_result.credits_used);
            remaining_credits = credit_result.remaining;
        }
    }

    // 3. Build response_format
    // If explicit response_format provided, use it. Otherwise if json_mode, default to json_object.
    let final_response_format = if input.response_format.is_some() {
        input.response_format.clone()
    } else if input.json_mode.unwrap_or(false) {
        Some(json!({ "type": "json_object" }))
    } else {
        None
    };

    // Determine json_mode flag for V5
    let json_mode = if input.json_mode.unwrap_or(false) {
        Some(true)
    } else if let Some(ref rf) = final_response_format {
        let rf_type = rf["type"].as_str().unwrap_or("");
        if rf_type == "json_object" || rf_type == "json_schema" {
            Some(true)
        } else {
            None
        }
    } else {
        None
    };

    // 4. Build V5 request
    let v5_req = V5Request {
        action: "generate_text".to_string(),
        model_code: input.model_code.unwrap_or_else(|| "gemini-2.5-pro".to_string()),
        user_id: user_id.clone(),
        prompt,
        system_prompt: input.system_prompt,
        max_tokens: input.max_tokens,
        temperature: input.temperature,
        json_mode,
        response_format: final_response_format,
        bypass_consume: true,
        attachments: input.attachments,
    };

    // 5. Call V5
    let v5_response = match call_v5(&v5_req).await {
        Ok(resp) => resp,
        Err(e) => {
            error!("[textgen] V5 call failed for user {}: {}", user_id, e);
            return json!({
                "success": false,
                "error": e,
                "metadata": {
                    "executionTime": start.elapsed().as_millis(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "creditsUsed": credits_used
                }
            });
        }
    };

    let v5_success = v5_response["success"].as_bool().unwrap_or(false);
    let v5_data = v5_response.get("data").cloned().unwrap_or(v5_response.clone());
    let v5_error = v5_response.get("error").and_then(|e| e.as_str()).map(|s| s.to_string());

    // 6. Auto-save result if requested
    let mut result_id: Option<String> = None;
    if let Some(ref save_opts) = input.save_result {
        if v5_success {
            let save_tool_id = save_opts
                .tool_id
                .as_deref()
                .or(input.tool_id.as_deref())
                .unwrap_or("unknown");
            result_id = save_result(&user_id, save_tool_id, &v5_data, save_opts.result_summary.as_deref()).await;
        }
    }

    // 7. Build response
    let elapsed = start.elapsed().as_millis();
    let timestamp = chrono::Utc::now().to_rfc3339();

    let mut metadata = json!({
        "executionTime": elapsed,
        "timestamp": timestamp
    });

    if let Some(cu) = credits_used {
        metadata["creditsUsed"] = json!(cu);
    }
    if let Some(rc) = remaining_credits {
        metadata["remainingCredits"] = json!(rc);
    }
    if let Some(ref rid) = result_id {
        metadata["resultId"] = json!(rid);
    }

    if v5_success {
        json!({
            "success": true,
            "data": v5_data,
            "metadata": metadata
        })
    } else {
        json!({
            "success": false,
            "data": v5_data,
            "error": v5_error.unwrap_or_else(|| "AI generation failed".to_string()),
            "metadata": metadata
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_textgen_input() {
        let input = json!({
            "action": "generate",
            "token": "test-token",
            "prompt": "Hello world",
            "model_code": "gemini-2.5-pro",
            "system_prompt": "You are a quiz master",
            "max_tokens": 4000,
            "json_mode": true,
            "toolId": "arena_quiz"
        });

        let parsed: TextGenInput = serde_json::from_value(input).unwrap();
        assert_eq!(parsed.action.as_deref(), Some("generate"));
        assert_eq!(parsed.prompt.as_deref(), Some("Hello world"));
        assert_eq!(parsed.model_code.as_deref(), Some("gemini-2.5-pro"));
        assert_eq!(parsed.system_prompt.as_deref(), Some("You are a quiz master"));
        assert_eq!(parsed.max_tokens, Some(4000));
        assert_eq!(parsed.json_mode, Some(true));
        assert_eq!(parsed.tool_id.as_deref(), Some("arena_quiz"));
    }

    #[test]
    fn test_parse_textgen_input_with_response_format() {
        let input = json!({
            "prompt": "Generate quiz",
            "token": "tok",
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": "quiz",
                    "schema": {
                        "type": "object",
                        "properties": {
                            "question": { "type": "string" },
                            "options": { "type": "array" }
                        }
                    }
                }
            }
        });

        let parsed: TextGenInput = serde_json::from_value(input).unwrap();
        assert!(parsed.response_format.is_some());
        let rf = parsed.response_format.unwrap();
        assert_eq!(rf["type"].as_str(), Some("json_schema"));
        assert!(rf["json_schema"]["schema"].is_object());
    }

    #[test]
    fn test_v5_request_serialization() {
        let req = V5Request {
            action: "generate_text".to_string(),
            model_code: "gemini-2.5-pro".to_string(),
            user_id: "user123".to_string(),
            prompt: "Test prompt".to_string(),
            system_prompt: Some("System".to_string()),
            max_tokens: Some(1000),
            temperature: None,
            json_mode: Some(true),
            response_format: Some(json!({ "type": "json_object" })),
            bypass_consume: true,
            attachments: None,
        };

        let serialized = serde_json::to_value(&req).unwrap();
        assert_eq!(serialized["action"], "generate_text");
        assert_eq!(serialized["bypassConsume"], true);
        assert_eq!(serialized["userId"], "user123");
        assert_eq!(serialized["json_mode"], true);
        assert!(serialized.get("temperature").is_none()); // skip_serializing_if None
        assert!(serialized.get("attachments").is_none());
    }

    #[test]
    fn test_v5_request_bypass_always_true() {
        let req = V5Request {
            action: "generate_text".to_string(),
            model_code: "gemini-2.5-flash".to_string(),
            user_id: "u1".to_string(),
            prompt: "Hi".to_string(),
            system_prompt: None,
            max_tokens: None,
            temperature: None,
            json_mode: None,
            response_format: None,
            bypass_consume: true,
            attachments: None,
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["bypassConsume"], true);
        // None fields should be absent
        assert!(json.get("system_prompt").is_none());
        assert!(json.get("max_tokens").is_none());
        assert!(json.get("json_mode").is_none());
        assert!(json.get("response_format").is_none());
    }

    #[tokio::test]
    async fn test_execute_missing_prompt() {
        let result = execute(json!({ "token": "test" })).await;
        assert_eq!(result["success"], false);
        assert!(result["error"].as_str().unwrap().contains("prompt"));
    }

    #[tokio::test]
    async fn test_execute_missing_token() {
        let result = execute(json!({ "prompt": "Hello" })).await;
        assert_eq!(result["success"], false);
        assert!(result["error"].as_str().unwrap().contains("token"));
    }

    #[tokio::test]
    async fn test_execute_invalid_token() {
        let result = execute(json!({
            "prompt": "Hello",
            "token": "invalid.jwt.token"
        }))
        .await;
        assert_eq!(result["success"], false);
        assert!(result["error"].as_str().unwrap().contains("token") ||
                result["error"].as_str().unwrap().contains("Invalid"));
    }
}