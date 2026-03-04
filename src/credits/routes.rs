//! Credit REST endpoints for Đấu Trường Vui
//!
//! All endpoints require `x-access-token` JWT header.
//! Data lives in PostgreSQL via PostgREST:
//!   - dtv_credit_wallets
//!   - dtv_credit_transactions
//!   - dtv_credit_usage
//!   - dtv_tool_settings
//!
//! Endpoints:
//!   POST /credits/wallet           -- get or create wallet
//!   POST /credits/deduct           -- deduct credits for tool usage
//!   POST /credits/claim-welcome-bonus -- one-time welcome bonus
//!   POST /credits/claim-daily-bonus   -- daily login bonus

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use tracing::{error, info, warn};

use crate::auth::middleware::AuthToken;

// ==================== Constants ====================

const WELCOME_BONUS_CREDITS: i64 = 50;
const DAILY_BONUS_CREDITS: i64 = 5;

// ==================== Types ====================

#[derive(Debug, Serialize)]
struct CreditResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    metadata: CreditMetadata,
}

#[derive(Debug, Serialize)]
struct CreditMetadata {
    #[serde(rename = "executionTime")]
    execution_time: u64,
    timestamp: String,
}

impl CreditResponse {
    fn ok(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: CreditMetadata {
                execution_time: 0,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        }
    }

    fn err(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
            metadata: CreditMetadata {
                execution_time: 0,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        }
    }

    fn with_time(mut self, ms: u64) -> Self {
        self.metadata.execution_time = ms;
        self
    }
}

impl IntoResponse for CreditResponse {
    fn into_response(self) -> Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };
        (status, Json(json!(self))).into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct DeductRequest {
    pub tool_id: Option<String>,
    pub amount: Option<i64>,
    #[allow(dead_code)]
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BonusRequest {
    #[allow(dead_code)]
    pub token: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "userSessionId")]
    pub user_session_id: Option<String>,
}

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

/// GET rows from PostgREST
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

/// POST (insert) a row into PostgREST, return created row
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

/// PATCH rows in PostgREST
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

// ==================== Router ====================

/// Build the credits sub-router. Mount at `/credits` in main.
pub fn credit_routes() -> Router {
    Router::new()
        .route("/wallet", post(wallet_handler))
        .route("/deduct", post(deduct_handler))
        .route("/claim-welcome-bonus", post(claim_welcome_bonus_handler))
        .route("/claim-daily-bonus", post(claim_daily_bonus_handler))
}

// ==================== Handlers ====================

/// POST /credits/wallet
///
/// Get the user's credit wallet. Auto-creates if not found.
/// Returns { paidCredits, referralCredits, bonusCredits, totalReferrals }
async fn wallet_handler(AuthToken(claims): AuthToken) -> CreditResponse {
    let start = std::time::Instant::now();
    let user_id = &claims.sub;

    info!("Credits wallet request for user {user_id}");

    // Query wallet
    let filter = format!("user_id=eq.{user_id}&select=paid_credits,referral_credits,bonus_credits,total_referrals");
    let wallets = match pg_get("credit_wallets", &filter).await {
        Ok(w) => w,
        Err(e) => {
            error!("Wallet query failed: {e}");
            return CreditResponse::err("Lỗi tải ví credits").with_time(start.elapsed().as_millis() as u64);
        }
    };

    let wallet = if let Some(w) = wallets.first() {
        w.clone()
    } else {
        // Auto-create wallet
        info!("Creating wallet for user {user_id}");
        let data = json!({
            "user_id": user_id,
            "paid_credits": 0,
            "referral_credits": 0,
            "bonus_credits": 0,
            "total_referrals": 0
        });
        match pg_insert("credit_wallets", &data).await {
            Ok(w) => w,
            Err(e) => {
                error!("Wallet creation failed: {e}");
                return CreditResponse::err("Lỗi tạo ví credits").with_time(start.elapsed().as_millis() as u64);
            }
        }
    };

    let paid = wallet["paid_credits"].as_i64().unwrap_or(0);
    let referral = wallet["referral_credits"].as_i64().unwrap_or(0);
    let bonus = wallet["bonus_credits"].as_i64().unwrap_or(0);
    let total_referrals = wallet["total_referrals"].as_i64().unwrap_or(0);

    CreditResponse::ok(json!({
        "paidCredits": paid,
        "referralCredits": referral,
        "bonusCredits": bonus,
        "totalReferrals": total_referrals,
        "total": paid + referral + bonus
    }))
    .with_time(start.elapsed().as_millis() as u64)
}

/// POST /credits/deduct
///
/// Deduct credits for using a tool.
/// Body: { tool_id: string, amount?: number }
/// If amount not provided, looks up cost from dtv_tool_settings.
async fn deduct_handler(
    AuthToken(claims): AuthToken,
    Json(body): Json<DeductRequest>,
) -> CreditResponse {
    let start = std::time::Instant::now();
    let user_id = &claims.sub;

    let tool_id = match &body.tool_id {
        Some(t) if !t.is_empty() => t.clone(),
        _ => {
            return CreditResponse::err("tool_id là bắt buộc")
                .with_time(start.elapsed().as_millis() as u64);
        }
    };

    info!("Credits deduct request: user={user_id} tool={tool_id}");

    // Determine cost
    let cost = if let Some(amount) = body.amount {
        if amount < 0 {
            return CreditResponse::err("Số lượng credits không hợp lệ")
                .with_time(start.elapsed().as_millis() as u64);
        }
        amount
    } else {
        // Look up from tool_settings
        match get_tool_cost(&tool_id).await {
            Ok(c) => c,
            Err(e) => {
                warn!("Tool cost lookup failed for {tool_id}: {e}");
                0 // Free if no setting found
            }
        }
    };

    // If cost is 0, check free daily limit
    if cost == 0 {
        match check_free_limit(user_id, &tool_id).await {
            Ok(true) => {
                // Within free limit, record usage and return
                let _ = record_usage(user_id, &tool_id).await;
                return CreditResponse::ok(json!({
                    "success": true,
                    "remaining": -1,
                    "cost": 0,
                    "message": "Miễn phí"
                }))
                .with_time(start.elapsed().as_millis() as u64);
            }
            Ok(false) => {
                // Exceeded free limit, need credits but cost is 0 from settings
                // This means the tool is truly free, allow anyway
                let _ = record_usage(user_id, &tool_id).await;
                return CreditResponse::ok(json!({
                    "success": true,
                    "remaining": -1,
                    "cost": 0,
                    "message": "Miễn phí"
                }))
                .with_time(start.elapsed().as_millis() as u64);
            }
            Err(e) => warn!("Free limit check failed: {e}"),
        }
    }

    // Get current wallet balance
    let filter = format!("user_id=eq.{user_id}&select=paid_credits,referral_credits,bonus_credits");
    let wallets = match pg_get("credit_wallets", &filter).await {
        Ok(w) => w,
        Err(e) => {
            error!("Wallet query for deduct failed: {e}");
            return CreditResponse::err("Lỗi tải ví credits")
                .with_time(start.elapsed().as_millis() as u64);
        }
    };

    let wallet = match wallets.first() {
        Some(w) => w,
        None => {
            return CreditResponse::err("Chưa có ví credits. Vui lòng tải lại trang.")
                .with_time(start.elapsed().as_millis() as u64);
        }
    };

    let paid = wallet["paid_credits"].as_i64().unwrap_or(0);
    let referral = wallet["referral_credits"].as_i64().unwrap_or(0);
    let bonus = wallet["bonus_credits"].as_i64().unwrap_or(0);
    let total = paid + referral + bonus;

    if total < cost {
        return CreditResponse::err(&format!(
            "Không đủ credits. Cần {cost}, hiện có {total}."
        ))
        .with_time(start.elapsed().as_millis() as u64);
    }

    // Deduct: bonus first, then referral, then paid
    let mut remaining_cost = cost;
    let mut new_bonus = bonus;
    let mut new_referral = referral;
    let mut new_paid = paid;

    if remaining_cost > 0 && new_bonus > 0 {
        let deduct_from_bonus = remaining_cost.min(new_bonus);
        new_bonus -= deduct_from_bonus;
        remaining_cost -= deduct_from_bonus;
    }
    if remaining_cost > 0 && new_referral > 0 {
        let deduct_from_referral = remaining_cost.min(new_referral);
        new_referral -= deduct_from_referral;
        remaining_cost -= deduct_from_referral;
    }
    if remaining_cost > 0 {
        new_paid -= remaining_cost;
    }

    // Update wallet
    let patch_filter = format!("user_id=eq.{user_id}");
    let patch_data = json!({
        "paid_credits": new_paid,
        "referral_credits": new_referral,
        "bonus_credits": new_bonus
    });

    if let Err(e) = pg_patch("credit_wallets", &patch_filter, &patch_data).await {
        error!("Wallet deduct patch failed: {e}");
        return CreditResponse::err("Lỗi trừ credits")
            .with_time(start.elapsed().as_millis() as u64);
    }

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

    // Record usage count
    let _ = record_usage(user_id, &tool_id).await;

    let new_total = new_paid + new_referral + new_bonus;
    info!("Credits deducted: user={user_id} tool={tool_id} cost={cost} remaining={new_total}");

    // Notify FE to refresh
    CreditResponse::ok(json!({
        "success": true,
        "remaining": new_total,
        "cost": cost,
        "paidCredits": new_paid,
        "referralCredits": new_referral,
        "bonusCredits": new_bonus
    }))
    .with_time(start.elapsed().as_millis() as u64)
}

/// POST /credits/claim-welcome-bonus
///
/// One-time welcome bonus for new users. Checks if already claimed.
async fn claim_welcome_bonus_handler(
    AuthToken(claims): AuthToken,
    Json(_body): Json<BonusRequest>,
) -> CreditResponse {
    let start = std::time::Instant::now();
    let user_id = &claims.sub;

    info!("Welcome bonus claim request: user={user_id}");

    // Check if already claimed
    let filter = format!(
        "user_id=eq.{user_id}&type=eq.welcome_bonus&select=id&limit=1"
    );
    let existing = match pg_get("credit_transactions", &filter).await {
        Ok(rows) => rows,
        Err(e) => {
            error!("Welcome bonus check failed: {e}");
            return CreditResponse::err("Lỗi kiểm tra bonus")
                .with_time(start.elapsed().as_millis() as u64);
        }
    };

    if !existing.is_empty() {
        return CreditResponse::ok(json!({
            "alreadyClaimed": true,
            "bonusCredits": 0,
            "message": "Bạn đã nhận bonus chào mừng rồi"
        }))
        .with_time(start.elapsed().as_millis() as u64);
    }

    // Grant bonus: add to bonus_credits in wallet
    let filter = format!("user_id=eq.{user_id}&select=bonus_credits");
    let wallets = match pg_get("credit_wallets", &filter).await {
        Ok(w) => w,
        Err(e) => {
            error!("Wallet query for welcome bonus failed: {e}");
            return CreditResponse::err("Lỗi tải ví credits")
                .with_time(start.elapsed().as_millis() as u64);
        }
    };

    let current_bonus = wallets
        .first()
        .and_then(|w| w["bonus_credits"].as_i64())
        .unwrap_or(0);

    let new_bonus = current_bonus + WELCOME_BONUS_CREDITS;
    let patch_filter = format!("user_id=eq.{user_id}");
    let patch_data = json!({ "bonus_credits": new_bonus });

    if let Err(e) = pg_patch("credit_wallets", &patch_filter, &patch_data).await {
        error!("Welcome bonus patch failed: {e}");
        return CreditResponse::err("Lỗi cấp bonus chào mừng")
            .with_time(start.elapsed().as_millis() as u64);
    }

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

    CreditResponse::ok(json!({
        "alreadyClaimed": false,
        "bonusCredits": WELCOME_BONUS_CREDITS,
        "totalBonusCredits": new_bonus,
        "message": format!("Chúc mừng! Bạn nhận được {WELCOME_BONUS_CREDITS} credits chào mừng!")
    }))
    .with_time(start.elapsed().as_millis() as u64)
}

/// POST /credits/claim-daily-bonus
///
/// Daily login bonus. One claim per calendar day.
async fn claim_daily_bonus_handler(
    AuthToken(claims): AuthToken,
    Json(_body): Json<BonusRequest>,
) -> CreditResponse {
    let start = std::time::Instant::now();
    let user_id = &claims.sub;

    info!("Daily bonus claim request: user={user_id}");

    // Check if already claimed today
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let filter = format!(
        "user_id=eq.{user_id}&type=eq.daily_bonus&created_at=gte.{today}T00:00:00Z&select=id&limit=1"
    );
    let existing = match pg_get("credit_transactions", &filter).await {
        Ok(rows) => rows,
        Err(e) => {
            error!("Daily bonus check failed: {e}");
            return CreditResponse::err("Lỗi kiểm tra bonus hàng ngày")
                .with_time(start.elapsed().as_millis() as u64);
        }
    };

    if !existing.is_empty() {
        return CreditResponse::ok(json!({
            "alreadyClaimed": true,
            "bonusCredits": 0,
            "message": "Bạn đã nhận bonus hôm nay rồi. Quay lại ngày mai nhé!"
        }))
        .with_time(start.elapsed().as_millis() as u64);
    }

    // Grant daily bonus
    let filter = format!("user_id=eq.{user_id}&select=bonus_credits");
    let wallets = match pg_get("credit_wallets", &filter).await {
        Ok(w) => w,
        Err(e) => {
            error!("Wallet query for daily bonus failed: {e}");
            return CreditResponse::err("Lỗi tải ví credits")
                .with_time(start.elapsed().as_millis() as u64);
        }
    };

    let current_bonus = wallets
        .first()
        .and_then(|w| w["bonus_credits"].as_i64())
        .unwrap_or(0);

    let new_bonus = current_bonus + DAILY_BONUS_CREDITS;
    let patch_filter = format!("user_id=eq.{user_id}");
    let patch_data = json!({ "bonus_credits": new_bonus });

    if let Err(e) = pg_patch("credit_wallets", &patch_filter, &patch_data).await {
        error!("Daily bonus patch failed: {e}");
        return CreditResponse::err("Lỗi cấp bonus hàng ngày")
            .with_time(start.elapsed().as_millis() as u64);
    }

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

    CreditResponse::ok(json!({
        "alreadyClaimed": false,
        "bonusCredits": DAILY_BONUS_CREDITS,
        "totalBonusCredits": new_bonus,
        "message": format!("Bạn nhận được {DAILY_BONUS_CREDITS} credits hàng ngày!")
    }))
    .with_time(start.elapsed().as_millis() as u64)
}

// ==================== Helper functions ====================

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
        None => Ok(0), // No setting = free
    }
}

/// Check if user is within free daily limit for a tool
async fn check_free_limit(user_id: &str, tool_id: &str) -> Result<bool, String> {
    // Get free_daily_limit from tool_settings
    let filter = format!("tool_id=eq.{tool_id}&select=free_daily_limit&limit=1");
    let settings = pg_get("tool_settings", &filter).await?;

    let free_limit = settings
        .first()
        .and_then(|s| s["free_daily_limit"].as_i64())
        .unwrap_or(0);

    if free_limit <= 0 {
        return Ok(false); // No free limit
    }

    // Get today's usage count
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

    // Check if row exists for today
    let filter = format!(
        "user_id=eq.{user_id}&tool_id=eq.{tool_id}&date=eq.{today}&select=id,count&limit=1"
    );
    let existing = pg_get("credit_usage", &filter).await?;

    if let Some(row) = existing.first() {
        // Increment existing
        let current = row["count"].as_i64().unwrap_or(0);
        let row_id = row["id"].as_str().unwrap_or("");
        if !row_id.is_empty() {
            let patch_filter = format!("id=eq.{row_id}");
            let patch_data = json!({ "count": current + 1 });
            pg_patch("credit_usage", &patch_filter, &patch_data).await?;
        }
    } else {
        // Insert new row
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
    fn test_credit_response_ok() {
        let resp = CreditResponse::ok(json!({"paidCredits": 100}));
        assert!(resp.success);
        assert!(resp.error.is_none());
        assert!(resp.data.is_some());
    }

    #[test]
    fn test_credit_response_err() {
        let resp = CreditResponse::err("Không đủ credits");
        assert!(!resp.success);
        assert_eq!(resp.error.as_deref(), Some("Không đủ credits"));
        assert!(resp.data.is_none());
    }

    #[test]
    fn test_credit_response_with_time() {
        let resp = CreditResponse::ok(json!({})).with_time(42);
        assert_eq!(resp.metadata.execution_time, 42);
    }

    #[test]
    fn test_table_name_prefix() {
        env::set_var("DB_TABLE_PREFIX", "dtv_");
        assert_eq!(table_name("credit_wallets"), "dtv_credit_wallets");
        assert_eq!(table_name("dtv_credit_wallets"), "dtv_credit_wallets");
    }

    #[test]
    fn test_deduct_request_deserialize() {
        let json_str = r#"{"tool_id": "arena_revive", "amount": 5, "token": "jwt..."}"#;
        let req: DeductRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(req.tool_id.as_deref(), Some("arena_revive"));
        assert_eq!(req.amount, Some(5));
    }

    #[test]
    fn test_deduct_request_minimal() {
        let json_str = r#"{"tool_id": "career_assessment"}"#;
        let req: DeductRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(req.tool_id.as_deref(), Some("career_assessment"));
        assert_eq!(req.amount, None);
    }

    #[test]
    fn test_bonus_request_deserialize() {
        let json_str = r#"{"token": "jwt...", "userSessionId": "sess-123"}"#;
        let req: BonusRequest = serde_json::from_str(json_str).unwrap();
        assert!(req.token.is_some());
        assert!(req.user_session_id.is_some());
    }

    #[test]
    fn test_bonus_request_empty() {
        let json_str = r#"{}"#;
        let req: BonusRequest = serde_json::from_str(json_str).unwrap();
        assert!(req.token.is_none());
    }

    #[test]
    fn test_credit_response_serialization() {
        let resp = CreditResponse::ok(json!({"total": 150}));
        let serialized = serde_json::to_string(&resp).unwrap();
        assert!(serialized.contains("\"success\":true"));
        assert!(serialized.contains("\"total\":150"));
        // error field should be absent (skip_serializing_if)
        assert!(!serialized.contains("\"error\""));
    }
}