//! Upload MCP tool for Đấu Trường Vui
//!
//! Wraps the existing S3 upload proxy logic as an MCP tool so FE can call via
//! `callTool('upload', { action: 'upload_files', token: '...', files: [...] })`
//! through `/tools/call`.
//!
//! Actions:
//!   upload_files -- upload base64-encoded files to S3 via MCP V5 proxy
//!
//! All actions require a valid JWT in `args.token`.
//! No AWS credentials in DTV BE -- V5 handles all S3 operations.
//!
//! Flow: FE -> DTV BE (tools/call name=upload) -> V5 (api_v5.ainext.vn/tools/s3_upload) -> AWS S3

use serde_json::{json, Value};
use std::env;
use std::sync::OnceLock;
use tracing::{error, info, warn};

use crate::auth::jwt;

// ==================== HTTP client singleton ====================

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn get_http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to build HTTP client for upload tool")
    })
}

// ==================== Config helpers ====================

fn v5_api_url() -> String {
    env::var("V5_API_URL").unwrap_or_else(|_| "http://api_v5.ainext.vn".to_string())
}

fn v5_api_key() -> Option<String> {
    env::var("V5_API_KEY").ok()
}

// ==================== Constants ====================

/// Max base64 content size per file (~10MB decoded ~ ~13.3MB base64)
const MAX_BASE64_SIZE: usize = 14_000_000;

/// Max number of files per request
const MAX_FILES_PER_REQUEST: usize = 10;

/// Fixed MongoDB ObjectId for V5 S3 uploads.
/// V5 requires a valid 24-char hex ObjectId when using API key auth.
/// DTV uses PostgreSQL UUIDs, not MongoDB -- so we use a shared service account ID.
const V5_SERVICE_USER_ID: &str = "000000000000000000000001";

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

/// Execute an upload tool action.
///
/// Called from `protocol_handler.rs` via `tools/call` with `name = "upload"`.
/// The `args` Value is the full `arguments` object from the JSON-RPC request.
pub async fn execute(args: Value) -> Value {
    let start = std::time::Instant::now();

    let action = args["action"].as_str().unwrap_or("");
    let token = args["token"].as_str().unwrap_or("");

    // All upload actions require a valid JWT
    if token.is_empty() {
        return err_response("Token là bắt buộc", 0);
    }

    let claims = match jwt::verify_jwt(token) {
        Ok(c) => c,
        Err(e) => {
            warn!("Upload tool: invalid token: {e}");
            return err_response("Token không hợp lệ hoặc đã hết hạn", 0);
        }
    };

    let user_id = claims.sub.clone();

    let result = match action {
        "upload_files" => handle_upload_files(&user_id, &args).await,
        "" => Err("action là bắt buộc".to_string()),
        other => Err(format!("Unknown upload action: {other}")),
    };

    let elapsed = start.elapsed().as_millis() as u64;

    match result {
        Ok(data) => ok_response(data, elapsed),
        Err(msg) => err_response(&msg, elapsed),
    }
}

// ==================== Action handlers ====================

/// Upload base64-encoded files to S3 via MCP V5 proxy.
///
/// Expects `args.files` as an array of `{ name, content, mimetype }` objects.
/// `content` can be raw base64 or a data URI (prefix will be stripped).
async fn handle_upload_files(user_id: &str, args: &Value) -> Result<Value, String> {
    let files = args["files"]
        .as_array()
        .ok_or_else(|| "files là bắt buộc (mảng các file)".to_string())?;

    if files.is_empty() {
        return Err("Không có file nào để tải lên".to_string());
    }

    if files.len() > MAX_FILES_PER_REQUEST {
        return Err(format!(
            "Tối đa {} file mỗi lần tải",
            MAX_FILES_PER_REQUEST
        ));
    }

    info!(
        "Upload tool: {} file(s) from user {}",
        files.len(),
        user_id
    );

    // Validate and clean files
    let mut cleaned_files: Vec<Value> = Vec::with_capacity(files.len());

    for (i, file) in files.iter().enumerate() {
        let name = file["name"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| format!("File #{} thiếu tên", i + 1))?;

        let content = file["content"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| format!("File '{}' không có nội dung", name))?;

        let mimetype = file["mimetype"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| format!("File '{}' thiếu mimetype", name))?;

        if content.len() > MAX_BASE64_SIZE {
            return Err(format!("File '{}' quá lớn (tối đa ~10MB)", name));
        }

        // Strip data URI prefix if present
        let clean_content = strip_data_uri_prefix(content);

        cleaned_files.push(json!({
            "name": name,
            "content": clean_content,
            "mimetype": mimetype,
        }));
    }

    // Check V5 API key
    let api_key = v5_api_key()
        .filter(|k| !k.is_empty())
        .ok_or_else(|| {
            error!("V5_API_KEY not configured");
            "Cấu hình server thiếu V5 API key".to_string()
        })?;

    // Build V5 request
    let v5_url = format!("{}/tools/s3_upload", v5_api_url());
    let v5_body = json!({
        "action": "upload",
        "files": cleaned_files,
        "userId": V5_SERVICE_USER_ID,
    });

    info!(
        "Upload tool: forwarding {} file(s) to V5: {}",
        cleaned_files.len(),
        v5_url
    );

    // Call V5
    let client = get_http_client();
    let v5_response = client
        .post(&v5_url)
        .header("X-API-Key", &api_key)
        .header("Content-Type", "application/json")
        .json(&v5_body)
        .send()
        .await
        .map_err(|e| {
            error!("V5 S3 upload request failed: {}", e);
            if e.is_timeout() {
                "Upload timeout -- thử lại với file nhỏ hơn".to_string()
            } else {
                format!("Lỗi kết nối V5: {}", e)
            }
        })?;

    let v5_status = v5_response.status();
    let v5_body_text = v5_response
        .text()
        .await
        .map_err(|e| {
            error!("Failed to read V5 response body: {}", e);
            "Lỗi đọc phản hồi từ V5".to_string()
        })?;

    if !v5_status.is_success() {
        let truncated = truncate(&v5_body_text, 200);
        warn!(
            "V5 S3 upload returned {} -- body: {}",
            v5_status, truncated
        );
        return Err(format!("V5 upload lỗi ({}): {}", v5_status, truncated));
    }

    // Parse V5 response
    let v5_result: Value = serde_json::from_str(&v5_body_text).map_err(|e| {
        error!("Failed to parse V5 response JSON: {}", e);
        "V5 trả về response không hợp lệ".to_string()
    })?;

    let success = v5_result["success"].as_bool().unwrap_or(false);
    if !success {
        let v5_error = v5_result["error"]
            .as_str()
            .unwrap_or("Upload thất bại");
        warn!("V5 upload returned success=false: {}", v5_error);
        return Err(v5_error.to_string());
    }

    // Extract uploaded file info
    let upload_data = v5_result["data"].clone();
    let uploaded_files = extract_uploaded_files(&upload_data);

    info!(
        "Upload tool: complete for user {} -- {} file(s) uploaded",
        user_id,
        uploaded_files.len()
    );

    Ok(json!({
        "uploadedFiles": uploaded_files,
        "totalFiles": uploaded_files.len(),
        "message": "Tải lên thành công"
    }))
}

// ==================== Helpers ====================

/// Strip data URI prefix: "data:image/jpeg;base64,/9j/..." -> "/9j/..."
fn strip_data_uri_prefix(content: &str) -> &str {
    if let Some(pos) = content.find(";base64,") {
        &content[pos + 8..]
    } else {
        content
    }
}

/// Truncate string for error messages
fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len]
    }
}

/// Extract uploaded file URLs from V5 response data.
/// V5 may return files in different structures depending on version.
fn extract_uploaded_files(data: &Value) -> Vec<Value> {
    // Try data.uploadedFiles (direct upload response)
    if let Some(files) = data["uploadedFiles"].as_array() {
        return files.clone();
    }

    // Try data.result.uploadedFiles (batch response)
    if let Some(files) = data["result"]["uploadedFiles"].as_array() {
        return files.clone();
    }

    // Try extracting from result.imageIds (batch studio style)
    if let Some(result) = data["result"].as_object() {
        if let Some(images) = result.get("imageIds") {
            if let Some(arr) = images.as_array() {
                return arr
                    .iter()
                    .filter_map(|img| {
                        Some(json!({
                            "url": img["image_url"].as_str()?,
                            "thumbUrl": img["thumb_url"].as_str().unwrap_or(""),
                            "mediaId": img["mediaId"].as_str().unwrap_or(""),
                        }))
                    })
                    .collect();
            }
        }
    }

    // Fallback: empty
    vec![]
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok_response_shape() {
        let resp = ok_response(json!({"uploadedFiles": [], "totalFiles": 0}), 100);
        assert_eq!(resp["success"], true);
        assert_eq!(resp["data"]["totalFiles"], 0);
        assert_eq!(resp["metadata"]["executionTime"], 100);
        assert!(resp["metadata"]["timestamp"].is_string());
    }

    #[test]
    fn test_err_response_shape() {
        let resp = err_response("Upload thất bại", 50);
        assert_eq!(resp["success"], false);
        assert_eq!(resp["error"], "Upload thất bại");
        assert_eq!(resp["metadata"]["executionTime"], 50);
    }

    #[test]
    fn test_strip_data_uri_prefix_jpeg() {
        let input = "data:image/jpeg;base64,/9j/4AAQSkZJRg==";
        assert_eq!(strip_data_uri_prefix(input), "/9j/4AAQSkZJRg==");
    }

    #[test]
    fn test_strip_data_uri_prefix_png() {
        let input = "data:image/png;base64,iVBORw0KGgo=";
        assert_eq!(strip_data_uri_prefix(input), "iVBORw0KGgo=");
    }

    #[test]
    fn test_strip_data_uri_prefix_no_prefix() {
        let input = "/9j/4AAQSkZJRg==";
        assert_eq!(strip_data_uri_prefix(input), "/9j/4AAQSkZJRg==");
    }

    #[test]
    fn test_strip_data_uri_prefix_empty() {
        assert_eq!(strip_data_uri_prefix(""), "");
    }

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long() {
        assert_eq!(truncate("hello world", 5), "hello");
    }

    #[test]
    fn test_extract_uploaded_files_direct() {
        let data = json!({
            "uploadedFiles": [
                { "url": "https://s3.example.com/a.jpg", "mediaId": "abc123" }
            ],
            "totalFiles": 1
        });
        let files = extract_uploaded_files(&data);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0]["url"], "https://s3.example.com/a.jpg");
    }

    #[test]
    fn test_extract_uploaded_files_nested() {
        let data = json!({
            "result": {
                "uploadedFiles": [
                    { "url": "https://s3.example.com/b.png", "mediaId": "def456" }
                ]
            }
        });
        let files = extract_uploaded_files(&data);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0]["url"], "https://s3.example.com/b.png");
    }

    #[test]
    fn test_extract_uploaded_files_image_ids() {
        let data = json!({
            "result": {
                "imageIds": [
                    {
                        "image_url": "https://api.aiva.vn/uploads/img1.jpg",
                        "thumb_url": "https://api.aiva.vn/uploads/img1_thumb.jpg",
                        "mediaId": "m1"
                    }
                ]
            }
        });
        let files = extract_uploaded_files(&data);
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0]["url"],
            "https://api.aiva.vn/uploads/img1.jpg"
        );
        assert_eq!(files[0]["mediaId"], "m1");
    }

    #[test]
    fn test_extract_uploaded_files_empty() {
        let data = json!({});
        let files = extract_uploaded_files(&data);
        assert!(files.is_empty());
    }

    #[tokio::test]
    async fn test_execute_missing_token() {
        let args = json!({"action": "upload_files", "files": []});
        let resp = execute(args).await;
        assert_eq!(resp["success"], false);
        assert!(resp["error"].as_str().unwrap().contains("Token"));
    }

    #[tokio::test]
    async fn test_execute_empty_token() {
        let args = json!({"action": "upload_files", "token": "", "files": []});
        let resp = execute(args).await;
        assert_eq!(resp["success"], false);
        assert!(resp["error"].as_str().unwrap().contains("Token"));
    }

    #[tokio::test]
    async fn test_execute_missing_action() {
        let args = json!({"token": "invalid"});
        let resp = execute(args).await;
        assert_eq!(resp["success"], false);
    }

    #[tokio::test]
    async fn test_execute_unknown_action() {
        let args = json!({"action": "delete_files", "token": "invalid"});
        let resp = execute(args).await;
        assert_eq!(resp["success"], false);
    }

    #[test]
    fn test_upload_files_args_parsing() {
        let args = json!({
            "action": "upload_files",
            "token": "jwt...",
            "files": [
                {
                    "name": "photo.jpg",
                    "content": "data:image/jpeg;base64,/9j/4AAQ",
                    "mimetype": "image/jpeg"
                }
            ]
        });
        let files = args["files"].as_array().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0]["name"], "photo.jpg");
        assert_eq!(files[0]["mimetype"], "image/jpeg");
    }

    #[test]
    fn test_upload_response_fields() {
        let data = json!({
            "uploadedFiles": [
                { "url": "https://api.aiva.vn/uploads/test.jpg" }
            ],
            "totalFiles": 1,
            "message": "Tải lên thành công"
        });
        let resp = ok_response(data, 200);
        assert_eq!(resp["data"]["totalFiles"], 1);
        assert_eq!(
            resp["data"]["uploadedFiles"][0]["url"],
            "https://api.aiva.vn/uploads/test.jpg"
        );
    }
}