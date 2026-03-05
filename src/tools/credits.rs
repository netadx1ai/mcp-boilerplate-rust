//! Credits MCP tool for Đấu Trường Vui
//!
//! Wraps the existing credit REST logic as an MCP tool so FE can call via
//! `callTool('credits', { action: '...', token: '...' })` through `/tools/call`.
//!
//! Actions:
//!   wallet             -- get or create wallet
//!   deduct             -- deduct credits for tool usage
//!   claim_welcome_bonus -- one-time welcome bonus
//!   claim_daily_bonus   -- daily login bonus
//!
//! All actions require a valid JWT in `args.token`.
//! Data lives in PostgreSQL via PostgREST:
//!   - dtv_credit_wallets
//!   - dtv_credit_transactions
//!   - dtv_credit_usage
//!   - dtv_tool_settings

use serde_json::{json, Value};
use std::env;
use tracing::{info, warn};

use crate::auth::jwt;

// ==================== Constants ====================

const WELCOME_BONUS_CREDITS: i64 = 50;
const DAILY_BONUS_CREDITS: i64 = 5;

// ==================== PostgREST helpers ====================

fn postgrest_url() -> String {
    env::var("POSTGREST_URL").unwrap_or_else(|_| "http://localhost:3001".to_string())
}

fn table_name(name: &str) -> String {
    let prefix = env::var("DB_TABLE_PREFIX").unwrap_or_else(|_| "dtv_".to_string());
    if name.starts_with(&prefix) {
        name.to_string()
    } else {
        format!("{prefix}{name}")
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

async fn pg_get(table: &str, query: &str) -> Result<Vec<Value>, String> {
    let url = format!("{}/{}?{}", postgrest_url(), table_name(table), query);
    let resp = client()
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("PostgREST GET failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("PostgREST GET error {status}: {body}"));
    }

    resp.json::<Vec<Value>>()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))
}

async fn pg_insert(table: &str, data: &Value) -> Result<Value, String> {
    let url = format!("{}/{}", postgrest_url(), table_name(table));
    let resp = client()
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Prefer", "return=representation")
        .json(data)
        .send()
        .await
        .map_err(|e| format!("PostgREST POST failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("PostgREST POST error {status}: {body}"));
    }

    let rows: Vec<Value> = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse insert response: {e}"))?;

    rows.into_iter()
        .next()
        .ok_or_else(|| "Insert returned no rows".to_string())
}

async fn pg_patch(table: &str, filter: &str, data: &Value) -> Result<Vec<Value>, String> {
    let url = format!("{}/{}?{}", postgrest_url(), table_name(table), filter);
    let resp = client()
        .patch(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Prefer", "return=representation")
        .json(data)
        .send()
        .await
        .map_err(|e| format!("PostgREST PATCH failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("PostgREST PATCH error {status}: {body}"));
    }

    resp.json::<Vec<Value>>()
        .await
        .map_err(|e| format!("Failed to parse patch response: {e}"))
}

// ==================== Response helpers ====================

fn ok_response(data: Value, elapsed_ms: u64) -> Value {
    json!({
        "success": true,
        "data": data,
        "metadata": {
            "executionTime": elapsed_ms,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    })
}

fn err_response(message: &str, elapsed_ms: u64) -> Value {
    json!({
        "success": false,
        "error": message,
        "metadata": {
            "executionTime": elapsed_ms,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    })
}

// ==================== Main entry point ====================

/// Execute a credits tool action.
///
/// Called from `protocol_handler.rs` via `tools/call` with `name = "credits"`.
/// The `args` Value is the full `arguments` object from the JSON-RPC request.
pub async fn execute(args: Value) -> Value {
    let start = std::time::Instant::now();

    let action = args["action"].as_str().unwrap_or("");
    let token = args["token"].as_str().unwrap_or("");

    // All credit actions require a valid JWT
    if token.is_empty() {
        return err_response("Token là bắt buộc", 0);
    }

    let claims = match jwt::verify_jwt(token) {
        Ok(c) => c,
        Err(e) => {
            warn!("Credits tool: invalid token: {e}");
            return err_response("Token không hợp lệ hoặc đã hết hạn", 0);
        }
    };

    let user_id = claims.sub.clone();

    let result = match action {
        "wallet" => handle_wallet(&user_id).await,
        "deduct" => handle_deduct(&user_id, &args).await,
        "claim_welcome_bonus" => handle_claim_welcome_bonus(&user_id).await,
        "claim_daily_bonus" => handle_claim_daily_bonus(&user_id).await,
        "" => Err("action là bắt buộc".to_string()),
        other => Err(format!("Unknown credits action: {other}")),
    };

    let elapsed = start.elapsed().as_millis() as u64;

    match result {
        Ok(data) => ok_response(data, elapsed),
        Err(msg) => err_response(&msg, elapsed),
    }
}

// ==================== Action handlers ====================

/// Get or create user's credit wallet.
async fn handle_wallet(user_id: &str) -> Result<Value, String> {
    info!("Credits tool: wallet for user {user_id}");

    let filter = format!(
        "user_id=eq.{user_id}&select=paid_credits,referral_credits,bonus_credits,total_referrals"
    );
    let wallets = pg_get("credit_wallets", &filter).await?;

    let wallet = if let Some(w) = wallets.first() {
        w.clone()
    } else {
        info!("Creating wallet for user {user_id}");
        let data = json!({
            "user_id": user_id,
            "paid_credits": 0,
            "referral_credits": 0,
            "bonus_credits": 0,
            "total_referrals": 0
        });
        pg_insert("credit_wallets", &data).await?
    };

    let paid = wallet["paid_credits"].as_i64().unwrap_or(0);
    let referral = wallet["referral_credits"].as_i64().unwrap_or(0);
    let bonus = wallet["bonus_credits"].as_i64().unwrap_or(0);
    let total_referrals = wallet["total_referrals"].as_i64().unwrap_or(0);

    Ok(json!({
        "paidCredits": paid,
        "referralCredits": referral,
        "bonusCredits": bonus,
        "totalReferrals": total_referrals,
        "total": paid + referral + bonus
    }))
}

/// Deduct credits for tool usage.
///
/// Expects `args.tool_id` (string) and optional `args.amount` (number).
/// If amount not provided, looks up cost from `dtv_tool_settings`.
async fn handle_deduct(user_id: &str, args: &Value) -> Result<Value, String> {
    let tool_id = args["tool_id"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "tool_id là bắt buộc".to_string())?;

    info!("Credits tool: deduct user={user_id} tool={tool_id}");

    // Determine cost
    let cost = if let Some(amount) = args["amount"].as_i64() {
        if amount < 0 {
            return Err("Số lượng credits không hợp lệ".to_string());
        }
        amount
    } else {
        get_tool_cost(tool_id).await.unwrap_or(0)
    };

    // If cost == 0, check free daily limit and allow
    if cost == 0 {
        match check_free_limit(user_id, tool_id).await {
            Ok(_within_limit) => {
                let _ = record_usage(user_id, tool_id).await;
                return Ok(json!({
                    "success": true,
                    "remaining": -1,
                    "cost": 0,
                    "message": "Miễn phí"
                }));
            }
            Err(e) => warn!("Free limit check failed: {e}"),
        }
    }

    // Get current wallet
    let filter = format!(
        "user_id=eq.{user_id}&select=paid_credits,referral_credits,bonus_credits"
    );
    let wallets = pg_get("credit_wallets", &filter).await?;

    let wallet = wallets
        .first()
        .ok_or_else(|| "Chưa có ví credits. Vui lòng tải lại trang.".to_string())?;

    let paid = wallet["paid_credits"].as_i64().unwrap_or(0);
    let referral = wallet["referral_credits"].as_i64().unwrap_or(0);
    let bonus = wallet["bonus_credits"].as_i64().unwrap_or(0);
    let total = paid + referral + bonus;

    if total < cost {
        return Err(format!(
            "Không đủ credits. Cần {cost}, hiện có {total}."
        ));
    }

    // Deduct: bonus first, then referral, then paid
    let mut remaining_cost = cost;
    let mut new_bonus = bonus;
    let mut new_referral = referral;
    let mut new_paid = paid;

    if remaining_cost > 0 && new_bonus > 0 {
        let d = remaining_cost.min(new_bonus);
        new_bonus -= d;
        remaining_cost -= d;
    }
    if remaining_cost > 0 && new_referral > 0 {
        let d = remaining_cost.min(new_referral);
        new_referral -= d;
        remaining_cost -= d;
    }
    if remaining_cost > 0 {
        new_paid -= remaining_cost;
    }

    // Patch wallet
    let patch_filter = format!("user_id=eq.{user_id}");
    let patch_data = json!({
        "paid_credits": new_paid,
        "referral_credits": new_referral,
        "bonus_credits": new_bonus
    });
    pg_patch("credit_wallets", &patch_filter, &patch_data).await?;

    // Record transaction
    let txn = json!({
        "user_id": user_id,
        "tool_id": tool_id,
        "amount": -cost,
        "type": "usage",
        "description": format!("Sử dụng {tool_id}")
    });
    if let Err(e) = pg_insert("credit_transactions", &txn).await {
        warn!("Transaction record failed (non-fatal): {e}");
    }

    // Record usage
    let _ = record_usage(user_id, tool_id).await;

    let new_total = new_paid + new_referral + new_bonus;
    info!("Credits deducted: user={user_id} tool={tool_id} cost={cost} remaining={new_total}");

    Ok(json!({
        "success": true,
        "remaining": new_total,
        "cost": cost,
        "paidCredits": new_paid,
        "referralCredits": new_referral,
        "bonusCredits": new_bonus
    }))
}

/// One-time welcome bonus for new users.
async fn handle_claim_welcome_bonus(user_id: &str) -> Result<Value, String> {
    info!("Credits tool: claim_welcome_bonus user={user_id}");

    // Check already claimed
    let filter = format!(
        "user_id=eq.{user_id}&type=eq.welcome_bonus&select=id&limit=1"
    );
    let existing = pg_get("credit_transactions", &filter).await?;

    if !existing.is_empty() {
        return Ok(json!({
            "alreadyClaimed": true,
            "bonusCredits": 0,
            "message": "Bạn đã nhận bonus chào mừng rồi"
        }));
    }

    // Get current bonus_credits
    let filter = format!("user_id=eq.{user_id}&select=bonus_credits");
    let wallets = pg_get("credit_wallets", &filter).await?;

    let current_bonus = wallets
        .first()
        .and_then(|w| w["bonus_credits"].as_i64())
        .unwrap_or(0);

    let new_bonus = current_bonus + WELCOME_BONUS_CREDITS;
    let patch_filter = format!("user_id=eq.{user_id}");
    let patch_data = json!({ "bonus_credits": new_bonus });
    pg_patch("credit_wallets", &patch_filter, &patch_data).await?;

    // Record transaction
    let txn = json!({
        "user_id": user_id,
        "tool_id": "welcome_bonus",
        "amount": WELCOME_BONUS_CREDITS,
        "type": "welcome_bonus",
        "description": format!("Bonus chào mừng: +{WELCOME_BONUS_CREDITS} credits")
    });
    if let Err(e) = pg_insert("credit_transactions", &txn).await {
        warn!("Welcome bonus transaction record failed: {e}");
    }

    info!("Welcome bonus granted: user={user_id} amount={WELCOME_BONUS_CREDITS}");

    Ok(json!({
        "alreadyClaimed": false,
        "bonusCredits": WELCOME_BONUS_CREDITS,
        "totalBonusCredits": new_bonus,
        "message": format!("Chúc mừng! Bạn nhận được {WELCOME_BONUS_CREDITS} credits chào mừng!")
    }))
}

/// Daily login bonus. One claim per calendar day.
async fn handle_claim_daily_bonus(user_id: &str) -> Result<Value, String> {
    info!("Credits tool: claim_daily_bonus user={user_id}");

    // Check already claimed today
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let filter = format!(
        "user_id=eq.{user_id}&type=eq.daily_bonus&created_at=gte.{today}T00:00:00Z&select=id&limit=1"
    );
    let existing = pg_get("credit_transactions", &filter).await?;

    if !existing.is_empty() {
        return Ok(json!({
            "alreadyClaimed": true,
            "bonusCredits": 0,
            "message": "Bạn đã nhận bonus hôm nay rồi. Quay lại ngày mai nhé!"
        }));
    }

    // Get current bonus_credits
    let filter = format!("user_id=eq.{user_id}&select=bonus_credits");
    let wallets = pg_get("credit_wallets", &filter).await?;

    let current_bonus = wallets
        .first()
        .and_then(|w| w["bonus_credits"].as_i64())
        .unwrap_or(0);

    let new_bonus = current_bonus + DAILY_BONUS_CREDITS;
    let patch_filter = format!("user_id=eq.{user_id}");
    let patch_data = json!({ "bonus_credits": new_bonus });
    pg_patch("credit_wallets", &patch_filter, &patch_data).await?;

    // Record transaction
    let txn = json!({
        "user_id": user_id,
        "tool_id": "daily_bonus",
        "amount": DAILY_BONUS_CREDITS,
        "type": "daily_bonus",
        "description": format!("Bonus hàng ngày: +{DAILY_BONUS_CREDITS} credits")
    });
    if let Err(e) = pg_insert("credit_transactions", &txn).await {
        warn!("Daily bonus transaction record failed: {e}");
    }

    info!("Daily bonus granted: user={user_id} amount={DAILY_BONUS_CREDITS}");

    Ok(json!({
        "alreadyClaimed": false,
        "bonusCredits": DAILY_BONUS_CREDITS,
        "totalBonusCredits": new_bonus,
        "message": format!("Bạn nhận được {DAILY_BONUS_CREDITS} credits hàng ngày!")
    }))
}

// ==================== Shared helpers ====================

/// Look up tool cost from dtv_tool_settings
async fn get_tool_cost(tool_id: &str) -> Result<i64, String> {
    let filter = format!("tool_id=eq.{tool_id}&select=cost,is_active&limit=1");
    let settings = pg_get("tool_settings", &filter).await?;

    match settings.first() {
        Some(s) => {
            let is_active = s["is_active"].as_bool().unwrap_or(true);
            if !is_active {
                return Err(format!("Tool {tool_id} is not active"));
            }
            Ok(s["cost"].as_i64().unwrap_or(0))
        }
        None => Ok(0),
    }
}

/// Check if user is within free daily limit for a tool
async fn check_free_limit(user_id: &str, tool_id: &str) -> Result<bool, String> {
    let filter = format!("tool_id=eq.{tool_id}&select=free_daily_limit&limit=1");
    let settings = pg_get("tool_settings", &filter).await?;

    let free_limit = settings
        .first()
        .and_then(|s| s["free_daily_limit"].as_i64())
        .unwrap_or(0);

    if free_limit <= 0 {
        return Ok(false);
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let filter = format!(
        "user_id=eq.{user_id}&tool_id=eq.{tool_id}&date=eq.{today}&select=count&limit=1"
    );
    let usage = pg_get("credit_usage", &filter).await?;

    let current_count = usage
        .first()
        .and_then(|u| u["count"].as_i64())
        .unwrap_or(0);

    Ok(current_count < free_limit)
}

/// Record a usage increment in dtv_credit_usage
async fn record_usage(user_id: &str, tool_id: &str) -> Result<(), String> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let filter = format!(
        "user_id=eq.{user_id}&tool_id=eq.{tool_id}&date=eq.{today}&select=id,count&limit=1"
    );
    let existing = pg_get("credit_usage", &filter).await?;

    if let Some(row) = existing.first() {
        let current = row["count"].as_i64().unwrap_or(0);
        let row_id = row["id"].as_str().unwrap_or("");
        if !row_id.is_empty() {
            let patch_filter = format!("id=eq.{row_id}");
            let patch_data = json!({ "count": current + 1 });
            pg_patch("credit_usage", &patch_filter, &patch_data).await?;
        }
    } else {
        let data = json!({
            "user_id": user_id,
            "tool_id": tool_id,
            "date": today,
            "count": 1
        });
        pg_insert("credit_usage", &data).await?;
    }

    Ok(())
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok_response_shape() {
        let resp = ok_response(json!({"paidCredits": 100}), 42);
        assert_eq!(resp["success"], true);
        assert_eq!(resp["data"]["paidCredits"], 100);
        assert_eq!(resp["metadata"]["executionTime"], 42);
        assert!(resp["metadata"]["timestamp"].is_string());
        // No error field when success
        assert!(resp.get("error").is_none());
    }

    #[test]
    fn test_err_response_shape() {
        let resp = err_response("Không đủ credits", 10);
        assert_eq!(resp["success"], false);
        assert_eq!(resp["error"], "Không đủ credits");
        assert_eq!(resp["metadata"]["executionTime"], 10);
        assert!(resp["metadata"]["timestamp"].is_string());
        // No data field when error
        assert!(resp.get("data").is_none());
    }

    #[test]
    fn test_table_name_prefix() {
        env::set_var("DB_TABLE_PREFIX", "dtv_");
        assert_eq!(table_name("credit_wallets"), "dtv_credit_wallets");
        assert_eq!(table_name("dtv_credit_wallets"), "dtv_credit_wallets");
    }

    #[tokio::test]
    async fn test_execute_missing_token() {
        let args = json!({"action": "wallet"});
        let resp = execute(args).await;
        assert_eq!(resp["success"], false);
        assert!(resp["error"].as_str().unwrap().contains("Token"));
    }

    #[tokio::test]
    async fn test_execute_missing_action() {
        // Token will fail verification, but empty action also checked
        let args = json!({"token": ""});
        let resp = execute(args).await;
        assert_eq!(resp["success"], false);
    }

    #[tokio::test]
    async fn test_execute_unknown_action() {
        // Will fail at token verification first (no real token)
        let args = json!({"action": "unknown_action", "token": "invalid"});
        let resp = execute(args).await;
        assert_eq!(resp["success"], false);
    }

    #[test]
    fn test_deduct_args_parsing() {
        let args = json!({
            "action": "deduct",
            "token": "jwt...",
            "tool_id": "arena_revive",
            "amount": 5
        });
        assert_eq!(args["tool_id"].as_str(), Some("arena_revive"));
        assert_eq!(args["amount"].as_i64(), Some(5));
    }

    #[test]
    fn test_deduct_args_minimal() {
        let args = json!({
            "action": "deduct",
            "token": "jwt...",
            "tool_id": "career_assessment"
        });
        assert_eq!(args["tool_id"].as_str(), Some("career_assessment"));
        assert!(args["amount"].as_i64().is_none());
    }

    #[test]
    fn test_wallet_response_fields() {
        let data = json!({
            "paidCredits": 100,
            "referralCredits": 20,
            "bonusCredits": 50,
            "totalReferrals": 3,
            "total": 170
        });
        let resp = ok_response(data, 5);
        assert_eq!(resp["data"]["paidCredits"], 100);
        assert_eq!(resp["data"]["referralCredits"], 20);
        assert_eq!(resp["data"]["bonusCredits"], 50);
        assert_eq!(resp["data"]["total"], 170);
    }

    #[test]
    fn test_deduct_response_fields() {
        let data = json!({
            "success": true,
            "remaining": 165,
            "cost": 5,
            "paidCredits": 95,
            "referralCredits": 20,
            "bonusCredits": 50
        });
        let resp = ok_response(data, 12);
        assert_eq!(resp["data"]["remaining"], 165);
        assert_eq!(resp["data"]["cost"], 5);
    }

    #[test]
    fn test_bonus_response_fields() {
        let data = json!({
            "alreadyClaimed": false,
            "bonusCredits": 50,
            "totalBonusCredits": 50,
            "message": "Chúc mừng!"
        });
        let resp = ok_response(data, 8);
        assert_eq!(resp["data"]["alreadyClaimed"], false);
        assert_eq!(resp["data"]["bonusCredits"], 50);
    }
}