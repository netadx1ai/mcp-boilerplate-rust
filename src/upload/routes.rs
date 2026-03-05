//! S3 Upload Proxy for Đấu Trường Vui
//!
//! Proxies file uploads to MCP V5 S3 upload endpoint.
//! No AWS credentials in DTV BE -- V5 handles all S3 operations.
//!
//! Flow: FE -> DTV BE (POST /upload) -> V5 (api_v5.ainext.vn/tools/s3_upload) -> AWS S3
//!
//! FE sends JSON with base64-encoded files (same pattern as admin-cms).
//! DTV BE forwards to V5 with `X-API-Key` header. Returns uploaded file URLs.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::sync::OnceLock;
use tracing::{error, info, warn};

use crate::auth::middleware::AuthToken;

// ---------------------------------------------------------------------------
// HTTP client singleton (60s timeout for uploads)
// ---------------------------------------------------------------------------

static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

fn get_http_client() -> &'static Client {
    HTTP_CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to build HTTP client for upload")
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

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Max base64 content size per file (~10MB decoded ≈ ~13.3MB base64)
const MAX_BASE64_SIZE: usize = 14_000_000;

/// Max number of files per request
const MAX_FILES_PER_REQUEST: usize = 10;

/// Fixed MongoDB ObjectId for V5 S3 uploads.
/// V5 requires a valid 24-char hex ObjectId when using API key auth.
/// DTV uses PostgreSQL UUIDs, not MongoDB -- so we use a shared service account ID.
/// This is only used for V5's internal file organization, not for DTV access control.
const V5_SERVICE_USER_ID: &str = "000000000000000000000001";

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct UploadFile {
    /// Original filename (e.g. "photo.jpg")
    pub name: String,
    /// Base64-encoded file content (data URI or raw base64)
    pub content: String,
    /// MIME type (e.g. "image/jpeg", "image/png")
    pub mimetype: String,
}

#[derive(Debug, Deserialize)]
pub struct UploadRequest {
    /// Files to upload (base64 encoded)
    pub files: Vec<UploadFile>,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// ---------------------------------------------------------------------------
// Upload handler
// ---------------------------------------------------------------------------

pub async fn handle_upload(
    auth: AuthToken,
    payload: UploadRequest,
) -> Response {
    let start = std::time::Instant::now();
    let AuthToken(claims) = auth;
    let user_id = claims.sub.clone();

    info!(
        "Upload request from user {} -- {} file(s)",
        user_id,
        payload.files.len()
    );

    // ---- Validate ----

    if payload.files.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Không có file nào để tải lên");
    }

    if payload.files.len() > MAX_FILES_PER_REQUEST {
        return error_response(
            StatusCode::BAD_REQUEST,
            &format!("Tối đa {} file mỗi lần tải", MAX_FILES_PER_REQUEST),
        );
    }

    for (i, file) in payload.files.iter().enumerate() {
        if file.name.is_empty() {
            return error_response(
                StatusCode::BAD_REQUEST,
                &format!("File #{} thiếu tên", i + 1),
            );
        }
        if file.content.is_empty() {
            return error_response(
                StatusCode::BAD_REQUEST,
                &format!("File '{}' không có nội dung", file.name),
            );
        }
        if file.mimetype.is_empty() {
            return error_response(
                StatusCode::BAD_REQUEST,
                &format!("File '{}' thiếu mimetype", file.name),
            );
        }
        if file.content.len() > MAX_BASE64_SIZE {
            return error_response(
                StatusCode::BAD_REQUEST,
                &format!(
                    "File '{}' quá lớn (tối đa ~10MB)",
                    file.name
                ),
            );
        }
    }

    // ---- Strip data URI prefix if present ----
    // FE FileReader.readAsDataURL() produces "data:image/jpeg;base64,/9j/4AAQ..."
    // V5 expects raw base64, so strip the prefix.

    let cleaned_files: Vec<Value> = payload
        .files
        .iter()
        .map(|f| {
            let content = strip_data_uri_prefix(&f.content);
            json!({
                "name": f.name,
                "content": content,
                "mimetype": f.mimetype,
            })
        })
        .collect();

    // ---- Check V5 API key ----

    let api_key = match v5_api_key() {
        Some(key) if !key.is_empty() => key,
        _ => {
            error!("V5_API_KEY not configured");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cấu hình server thiếu V5 API key",
            );
        }
    };

    // ---- Build V5 request ----
    // V5 requires userId as a MongoDB ObjectId (24-char hex) when using API key auth.
    // DTV users have PostgreSQL UUIDs, so we use a fixed service ObjectId.

    let v5_url = format!("{}/tools/s3_upload", v5_api_url());
    let v5_body = json!({
        "action": "upload",
        "files": cleaned_files,
        "userId": V5_SERVICE_USER_ID,
    });

    info!(
        "Forwarding {} file(s) to V5: {}",
        cleaned_files.len(),
        v5_url
    );

    // ---- Call V5 ----

    let client = get_http_client();
    let v5_response = match client
        .post(&v5_url)
        .header("X-API-Key", &api_key)
        .header("Content-Type", "application/json")
        .json(&v5_body)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            error!("V5 S3 upload request failed: {}", e);
            let msg = if e.is_timeout() {
                "Upload timeout -- thử lại với file nhỏ hơn".to_string()
            } else {
                format!("Lỗi kết nối V5: {}", e)
            };
            // Return 200 with success=false to avoid Cloudflare intercepting 502/504
            return ok_error(&msg);
        }
    };

    let v5_status = v5_response.status();
    let v5_body_text = match v5_response.text().await {
        Ok(text) => text,
        Err(e) => {
            error!("Failed to read V5 response body: {}", e);
            return ok_error("Lỗi đọc phản hồi từ V5");
        }
    };

    if !v5_status.is_success() {
        warn!(
            "V5 S3 upload returned {} -- body: {}",
            v5_status,
            &v5_body_text[..v5_body_text.len().min(500)]
        );
        return ok_error(&format!(
            "V5 upload lỗi ({}): {}",
            v5_status,
            truncate(&v5_body_text, 200)
        ));
    }

    // ---- Parse V5 response ----

    let v5_result: Value = match serde_json::from_str(&v5_body_text) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to parse V5 response JSON: {}", e);
            return error_response(
                StatusCode::BAD_GATEWAY,
                "V5 trả về response không hợp lệ",
            );
        }
    };

    let elapsed = start.elapsed().as_millis();

    // V5 response structure: { success, message, data: { result: { ... }, errors: [] } }
    // or { success, message, data: { uploadedFiles: [...], totalFiles, ... } }
    // We normalize and pass through.

    let success = v5_result["success"].as_bool().unwrap_or(false);

    if !success {
        let v5_error = v5_result["error"]
            .as_str()
            .unwrap_or("Upload thất bại");
        warn!("V5 upload returned success=false: {}", v5_error);
        return (
            StatusCode::OK,
            Json(UploadResponse {
                success: false,
                data: None,
                error: Some(v5_error.to_string()),
                message: None,
            }),
        )
            .into_response();
    }

    // Extract uploaded file info -- V5 may nest under data.result or data directly
    let upload_data = if v5_result["data"]["result"].is_object() || v5_result["data"]["result"].is_array() {
        // Batch studio style response
        v5_result["data"].clone()
    } else {
        v5_result["data"].clone()
    };

    // Build a flat list of uploaded file URLs for easy FE consumption
    let uploaded_files = extract_uploaded_files(&upload_data);

    info!(
        "Upload complete for user {} -- {} file(s) uploaded in {}ms",
        user_id,
        uploaded_files.len(),
        elapsed
    );

    (
        StatusCode::OK,
        Json(UploadResponse {
            success: true,
            data: Some(json!({
                "uploadedFiles": uploaded_files,
                "totalFiles": uploaded_files.len(),
                "v5Data": upload_data,
            })),
            error: None,
            message: Some("Tải lên thành công".to_string()),
        }),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

/// Extract uploaded file URLs from V5 response data
fn extract_uploaded_files(data: &Value) -> Vec<Value> {
    // Try data.uploadedFiles (direct upload response)
    if let Some(files) = data["uploadedFiles"].as_array() {
        return files.clone();
    }

    // Try data.result.uploadedFiles (batch response)
    if let Some(files) = data["result"]["uploadedFiles"].as_array() {
        return files.clone();
    }

    // Try extracting from result.imageIds / result (batch studio doc)
    if let Some(result) = data["result"].as_object() {
        // If result has image_ids or similar, build a simplified response
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

    // Fallback: return empty
    vec![]
}

/// Build an error response with a specific HTTP status code (for client errors)
fn error_response(status: StatusCode, message: &str) -> Response {
    (
        status,
        Json(UploadResponse {
            success: false,
            data: None,
            error: Some(message.to_string()),
            message: None,
        }),
    )
        .into_response()
}

/// Build an error response with HTTP 200 but success=false.
/// Used for V5 upstream errors to avoid Cloudflare intercepting 502/504.
fn ok_error(message: &str) -> Response {
    (
        StatusCode::OK,
        Json(UploadResponse {
            success: false,
            data: None,
            error: Some(message.to_string()),
            message: None,
        }),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_data_uri_prefix_with_prefix() {
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
    fn test_extract_uploaded_files_empty() {
        let data = json!({});
        let files = extract_uploaded_files(&data);
        assert!(files.is_empty());
    }

    #[test]
    fn test_upload_request_deserialization() {
        let json_str = r#"{
            "files": [{
                "name": "test.jpg",
                "content": "base64data",
                "mimetype": "image/jpeg"
            }]
        }"#;
        let req: UploadRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(req.files.len(), 1);
        assert_eq!(req.files[0].name, "test.jpg");
        assert_eq!(req.files[0].mimetype, "image/jpeg");
    }

    #[test]
    fn test_upload_response_serialization() {
        let resp = UploadResponse {
            success: true,
            data: Some(json!({"uploadedFiles": []})),
            error: None,
            message: Some("OK".to_string()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_upload_response_error_serialization() {
        let resp = UploadResponse {
            success: false,
            data: None,
            error: Some("Upload failed".to_string()),
            message: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("Upload failed"));
        assert!(!json.contains("\"data\""));
        assert!(!json.contains("\"message\""));
    }
}