//! PostgreSQL Database Tool via PostgREST Wrapper
//!
//! Translates MCP tool calls into PostgREST HTTP requests.
//! Actions: query, insert, update, delete, upsert, rpc, list_tables, describe.

use chrono::Utc;
use reqwest::{header::HeaderMap, Client, Method, StatusCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PostgRestConfig {
    pub base_url: String,
    pub anon_key: Option<String>,
    pub timeout_secs: u64,
    pub allowed_tables: Option<HashSet<String>>,
    pub table_prefix: Option<String>,
}

impl PostgRestConfig {
    pub fn from_env() -> Self {
        let base_url = std::env::var("POSTGREST_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string())
            .trim_end_matches('/')
            .to_string();

        let anon_key = std::env::var("POSTGREST_ANON_KEY").ok().filter(|k| !k.is_empty());

        let timeout_secs = std::env::var("POSTGREST_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30u64);

        let allowed_tables = std::env::var("DB_ALLOWED_TABLES").ok().map(|v| {
            v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<HashSet<String>>()
        });

        let table_prefix = std::env::var("DB_TABLE_PREFIX")
            .ok()
            .filter(|p| !p.is_empty());

        Self {
            base_url,
            anon_key,
            timeout_secs,
            allowed_tables,
            table_prefix,
        }
    }

    /// Check if a table is allowed by whitelist and/or prefix.
    /// If neither is configured, all tables are allowed.
    pub fn is_table_allowed(&self, table: &str) -> bool {
        let has_whitelist = self.allowed_tables.as_ref().is_some_and(|s| !s.is_empty());
        let has_prefix = self.table_prefix.is_some();

        if !has_whitelist && !has_prefix {
            return true;
        }

        if has_whitelist {
            if let Some(ref set) = self.allowed_tables {
                if set.contains(table) {
                    return true;
                }
            }
        }

        if let Some(ref prefix) = self.table_prefix {
            if table.starts_with(prefix.as_str()) {
                return true;
            }
        }

        false
    }
}

/// Validate table name to prevent path traversal / injection.
/// Only allows `[a-zA-Z_][a-zA-Z0-9_]*`.
pub fn validate_table_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Table name cannot be empty".to_string());
    }
    let first = name.as_bytes()[0];
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return Err(format!(
            "Invalid table name '{name}': must start with a letter or underscore"
        ));
    }
    if !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(format!(
            "Invalid table name '{name}': only alphanumeric and underscore allowed"
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DbRequest {
    /// Action to perform (query, insert, update, delete, upsert, rpc, list_tables, describe)
    pub action: String,

    /// JWT token for PostgREST authorization (overrides anon key)
    #[serde(default)]
    pub token: Option<String>,

    /// Target table name
    #[serde(default)]
    pub table: Option<String>,

    /// Columns to select (string "id,name" or array ["id","name"])
    #[serde(default)]
    pub select: Option<Value>,

    /// Filter conditions: { "col": { "op": value } }
    #[serde(default)]
    pub filters: Option<Value>,

    /// Data payload for insert/update/upsert (object or array of objects)
    #[serde(default)]
    pub data: Option<Value>,

    /// Order specification: [{ "column": "name", "direction": "asc" }]
    #[serde(default)]
    pub order: Option<Value>,

    /// Maximum number of rows to return
    #[serde(default)]
    pub limit: Option<u64>,

    /// Number of rows to skip
    #[serde(default)]
    pub offset: Option<u64>,

    /// Additional options (count, single, return)
    #[serde(default)]
    pub options: Option<DbOptions>,

    /// Function name for rpc action
    #[serde(default, alias = "function")]
    pub function_name: Option<String>,

    /// Parameters for rpc action
    #[serde(default)]
    pub params: Option<Value>,

    /// Conflict columns for upsert (comma-separated string)
    #[serde(default)]
    pub conflict: Option<String>,

    /// Raw SQL (not supported in PostgREST mode)
    #[serde(default)]
    pub sql: Option<String>,

    /// SQL parameters (not supported in PostgREST mode)
    #[serde(default, alias = "sqlParams")]
    pub sql_params: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Default)]
pub struct DbOptions {
    /// Set to "exact" to get total count via Content-Range
    #[serde(default)]
    pub count: Option<String>,

    /// Set to true to return a single object instead of array
    #[serde(default)]
    pub single: Option<bool>,

    /// Set to "minimal" to skip returning data on write ops
    #[serde(default, alias = "return")]
    pub return_pref: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DbResponse {
    pub success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,

    pub metadata: DbMetadata,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DbMetadata {
    pub execution_time_ms: u64,
    pub timestamp: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_rows: Option<usize>,
}

impl DbResponse {
    pub fn ok(
        data: Option<Value>,
        count: Option<i64>,
        affected_rows: Option<usize>,
        action: &str,
        table: Option<&str>,
        start: Instant,
    ) -> Self {
        Self {
            success: true,
            data,
            error: None,
            count,
            metadata: DbMetadata {
                execution_time_ms: start.elapsed().as_millis() as u64,
                timestamp: Utc::now().to_rfc3339(),
                action: Some(action.to_string()),
                table: table.map(|s| s.to_string()),
                affected_rows,
            },
        }
    }

    pub fn err(msg: impl Into<String>, action: &str, table: Option<&str>, start: Instant) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
            count: None,
            metadata: DbMetadata {
                execution_time_ms: start.elapsed().as_millis() as u64,
                timestamp: Utc::now().to_rfc3339(),
                action: Some(action.to_string()),
                table: table.map(|s| s.to_string()),
                affected_rows: None,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Filter Translation
// ---------------------------------------------------------------------------

/// Translate MCP filter JSON -> PostgREST query params
/// Supports: eq, neq, gt, gte, lt, lte, like, ilike, is, in, not, contains, containedBy, overlaps
pub fn translate_filters(filters: &Value) -> Result<Vec<(String, String)>, String> {
    let obj = match filters.as_object() {
        Some(o) => o,
        None => return Err("Filters must be a JSON object".to_string()),
    };

    // Detect legacy format: { "eq": { "col": val } }
    let known_ops: HashSet<&str> = [
        "eq",
        "neq",
        "gt",
        "gte",
        "lt",
        "lte",
        "like",
        "ilike",
        "is",
        "in",
        "not",
        "contains",
        "containedBy",
        "overlaps",
    ]
    .into_iter()
    .collect();

    // Check if top-level keys are all operators (legacy format)
    let all_operators = !obj.is_empty() && obj.keys().all(|k| known_ops.contains(k.as_str()));

    if all_operators {
        // Legacy: { "eq": { "col": val, ... }, "gt": { "col2": val2 } }
        return translate_legacy_filters(obj, &known_ops);
    }

    // Standard format: { "col": { "op": val } } or { "col": val } (shorthand eq)
    let mut params = Vec::new();
    for (col, val) in obj {
        if val.is_null() {
            // { "col": null } -> col=is.null
            params.push((col.clone(), "is.null".to_string()));
        } else if val.is_object() {
            let ops = val.as_object().unwrap();
            for (op, v) in ops {
                let pg_val = format_postgrest_value(op.as_str(), v)?;
                params.push((col.clone(), pg_val));
            }
        } else {
            // Simple equality: { "col": "val" } -> col=eq.val
            params.push((col.clone(), format!("eq.{}", value_to_string(val))));
        }
    }
    Ok(params)
}

fn translate_legacy_filters(
    obj: &serde_json::Map<String, Value>,
    _known_ops: &HashSet<&str>,
) -> Result<Vec<(String, String)>, String> {
    let mut params = Vec::new();
    for (op, columns) in obj {
        let cols = columns
            .as_object()
            .ok_or_else(|| format!("Legacy filter operator '{op}' value must be an object"))?;
        for (col, val) in cols {
            let pg_val = format_postgrest_value(op.as_str(), val)?;
            params.push((col.clone(), pg_val));
        }
    }
    Ok(params)
}

fn format_postgrest_value(op: &str, val: &Value) -> Result<String, String> {
    match op {
        "eq" => Ok(format!("eq.{}", value_to_string(val))),
        "neq" => Ok(format!("neq.{}", value_to_string(val))),
        "gt" => Ok(format!("gt.{}", value_to_string(val))),
        "gte" => Ok(format!("gte.{}", value_to_string(val))),
        "lt" => Ok(format!("lt.{}", value_to_string(val))),
        "lte" => Ok(format!("lte.{}", value_to_string(val))),
        "like" => {
            let s = value_to_string(val).replace('%', "*");
            Ok(format!("like.{s}"))
        }
        "ilike" => {
            let s = value_to_string(val).replace('%', "*");
            Ok(format!("ilike.{s}"))
        }
        "is" => {
            let s = if val.is_null() {
                "null".to_string()
            } else {
                value_to_string(val)
            };
            Ok(format!("is.{s}"))
        }
        "in" => {
            let arr = val
                .as_array()
                .ok_or_else(|| "'in' filter value must be an array".to_string())?;
            let csv = arr.iter().map(value_to_string).collect::<Vec<_>>().join(",");
            Ok(format!("in.({csv})"))
        }
        "not" => Ok(format!("not.eq.{}", value_to_string(val))),
        "contains" => {
            let s = format_array_literal(val)?;
            Ok(format!("cs.{s}"))
        }
        "containedBy" => {
            let s = format_array_literal(val)?;
            Ok(format!("cd.{s}"))
        }
        "overlaps" => {
            let s = format_array_literal(val)?;
            Ok(format!("ov.{s}"))
        }
        other => Err(format!("Unknown filter operator: '{other}'")),
    }
}

fn format_array_literal(val: &Value) -> Result<String, String> {
    let arr = val
        .as_array()
        .ok_or_else(|| "Array operator value must be an array".to_string())?;
    let csv = arr.iter().map(value_to_string).collect::<Vec<_>>().join(",");
    Ok(format!("{{{csv}}}"))
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Order / Select helpers
// ---------------------------------------------------------------------------

pub fn translate_order(order: &Value) -> Result<String, String> {
    match order {
        Value::String(s) => Ok(s.clone()),
        Value::Array(arr) => {
            let parts: Result<Vec<String>, String> = arr
                .iter()
                .map(|item| {
                    let obj = item
                        .as_object()
                        .ok_or_else(|| "Order item must be an object".to_string())?;
                    let col = obj
                        .get("column")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "Order item must have 'column' field".to_string())?;
                    let dir = if let Some(d) = obj.get("direction").and_then(|v| v.as_str()) {
                        d.to_string()
                    } else if let Some(asc) = obj.get("ascending").and_then(|v| v.as_bool()) {
                        if asc { "asc" } else { "desc" }.to_string()
                    } else {
                        "asc".to_string()
                    };
                    Ok(format!("{col}.{dir}"))
                })
                .collect();
            Ok(parts?.join(","))
        }
        _ => Err("Order must be a string or array".to_string()),
    }
}

pub fn translate_select(select: &Value) -> Result<String, String> {
    match select {
        Value::String(s) => Ok(s.clone()),
        Value::Array(arr) => {
            let cols: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if cols.is_empty() {
                return Err("Select array must contain string column names".to_string());
            }
            Ok(cols.join(","))
        }
        _ => Err("Select must be a string or array".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Request Builder
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct PostgRestRequest {
    pub method: Method,
    pub path: String,
    pub query_params: Vec<(String, String)>,
    pub headers: HeaderMap,
    pub body: Option<Value>,
}

pub fn build_request(
    req: &DbRequest,
    config: &PostgRestConfig,
) -> Result<PostgRestRequest, String> {
    let action = req.action.to_lowercase();
    let table = req.table.as_deref();

    // Validate table when required
    let needs_table = matches!(
        action.as_str(),
        "query" | "select" | "insert" | "create" | "update" | "delete" | "remove" | "upsert" | "describe" | "schema"
    );
    if needs_table {
        let t = table.ok_or_else(|| format!("Action '{action}' requires 'table' field"))?;
        validate_table_name(t)?;
        if !config.is_table_allowed(t) {
            return Err(format!("Table '{t}' is not in the allowed tables list"));
        }
    }

    match action.as_str() {
        "query" | "select" => build_query_request(req, config),
        "insert" | "create" => build_insert_request(req, config),
        "update" => build_update_request(req, config),
        "delete" | "remove" => build_delete_request(req, config),
        "upsert" => build_upsert_request(req, config),
        "rpc" | "function" | "call" => build_rpc_request(req, config),
        "list_tables" | "tables" => build_list_tables_request(config),
        "describe" | "schema" => build_describe_request(req, config),
        "raw_sql" | "sql" => Err(
            "raw_sql is not supported in PostgREST mode. \
             Use 'rpc' action with a PostgreSQL function instead. \
             Example: { \"action\": \"rpc\", \"function_name\": \"my_func\", \"params\": {...} }"
                .to_string(),
        ),
        _ => Err(format!(
            "Unknown action '{action}'. Valid actions: query, insert, update, delete, \
             upsert, rpc, list_tables, describe"
        )),
    }
}

fn base_headers(req: &DbRequest, config: &PostgRestConfig) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    // Auth: token overrides anon key
    if let Some(ref token) = req.token {
        headers.insert(
            "Authorization",
            format!("Bearer {token}").parse().unwrap(),
        );
    } else if let Some(ref key) = config.anon_key {
        headers.insert("Authorization", format!("Bearer {key}").parse().unwrap());
    }

    // Options
    if let Some(ref opts) = req.options {
        if opts.single == Some(true) {
            headers.insert(
                "Accept",
                "application/vnd.pgrst.object+json".parse().unwrap(),
            );
        }
    }

    headers
}

fn add_prefer_headers(headers: &mut HeaderMap, req: &DbRequest, default_return: &str) {
    let mut prefs = Vec::new();

    let return_pref = req
        .options
        .as_ref()
        .and_then(|o| o.return_pref.as_deref())
        .unwrap_or(default_return);
    prefs.push(format!("return={return_pref}"));

    if let Some(ref opts) = req.options {
        if opts.count.as_deref() == Some("exact") {
            prefs.push("count=exact".to_string());
        }
    }

    if !prefs.is_empty() {
        headers.insert("Prefer", prefs.join(",").parse().unwrap());
    }
}

fn build_query_request(
    req: &DbRequest,
    config: &PostgRestConfig,
) -> Result<PostgRestRequest, String> {
    let table = req.table.as_deref().unwrap();
    let mut qp = Vec::new();

    if let Some(ref sel) = req.select {
        qp.push(("select".to_string(), translate_select(sel)?));
    }
    if let Some(ref filters) = req.filters {
        qp.extend(translate_filters(filters)?);
    }
    if let Some(ref order) = req.order {
        qp.push(("order".to_string(), translate_order(order)?));
    }
    if let Some(limit) = req.limit {
        qp.push(("limit".to_string(), limit.to_string()));
    }
    if let Some(offset) = req.offset {
        qp.push(("offset".to_string(), offset.to_string()));
    }

    let mut headers = base_headers(req, config);
    // Add count header if requested
    if let Some(ref opts) = req.options {
        if opts.count.as_deref() == Some("exact") {
            headers.insert("Prefer", "count=exact".parse().unwrap());
        }
    }

    Ok(PostgRestRequest {
        method: Method::GET,
        path: format!("{}/{table}", config.base_url),
        query_params: qp,
        headers,
        body: None,
    })
}

fn build_insert_request(
    req: &DbRequest,
    config: &PostgRestConfig,
) -> Result<PostgRestRequest, String> {
    let table = req.table.as_deref().unwrap();
    let data = req
        .data
        .as_ref()
        .ok_or_else(|| "Action 'insert' requires 'data' field".to_string())?;

    let mut headers = base_headers(req, config);
    add_prefer_headers(&mut headers, req, "representation");

    Ok(PostgRestRequest {
        method: Method::POST,
        path: format!("{}/{table}", config.base_url),
        query_params: Vec::new(),
        headers,
        body: Some(data.clone()),
    })
}

fn build_update_request(
    req: &DbRequest,
    config: &PostgRestConfig,
) -> Result<PostgRestRequest, String> {
    let table = req.table.as_deref().unwrap();
    let data = req
        .data
        .as_ref()
        .ok_or_else(|| "Action 'update' requires 'data' field".to_string())?;

    if req.filters.is_none() {
        return Err(
            "Action 'update' requires 'filters' to prevent mass updates. \
             Use filters to specify which rows to update."
                .to_string(),
        );
    }

    let mut qp = Vec::new();
    if let Some(ref filters) = req.filters {
        qp.extend(translate_filters(filters)?);
    }

    let mut headers = base_headers(req, config);
    add_prefer_headers(&mut headers, req, "representation");

    Ok(PostgRestRequest {
        method: Method::PATCH,
        path: format!("{}/{table}", config.base_url),
        query_params: qp,
        headers,
        body: Some(data.clone()),
    })
}

fn build_delete_request(
    req: &DbRequest,
    config: &PostgRestConfig,
) -> Result<PostgRestRequest, String> {
    let table = req.table.as_deref().unwrap();

    if req.filters.is_none() {
        return Err(
            "Action 'delete' requires 'filters' to prevent mass deletes. \
             Use filters to specify which rows to delete."
                .to_string(),
        );
    }

    let mut qp = Vec::new();
    if let Some(ref filters) = req.filters {
        qp.extend(translate_filters(filters)?);
    }

    let mut headers = base_headers(req, config);
    add_prefer_headers(&mut headers, req, "representation");

    Ok(PostgRestRequest {
        method: Method::DELETE,
        path: format!("{}/{table}", config.base_url),
        query_params: qp,
        headers,
        body: None,
    })
}

fn build_upsert_request(
    req: &DbRequest,
    config: &PostgRestConfig,
) -> Result<PostgRestRequest, String> {
    let table = req.table.as_deref().unwrap();
    let data = req
        .data
        .as_ref()
        .ok_or_else(|| "Action 'upsert' requires 'data' field".to_string())?;

    let mut qp = Vec::new();
    if let Some(ref conflict) = req.conflict {
        qp.push(("on_conflict".to_string(), conflict.clone()));
    }

    let mut headers = base_headers(req, config);
    // Upsert uses resolution=merge-duplicates
    let mut prefs = vec!["return=representation".to_string(), "resolution=merge-duplicates".to_string()];
    if let Some(ref opts) = req.options {
        if opts.count.as_deref() == Some("exact") {
            prefs.push("count=exact".to_string());
        }
        if let Some(ref rp) = opts.return_pref {
            prefs[0] = format!("return={rp}");
        }
    }
    headers.insert("Prefer", prefs.join(",").parse().unwrap());

    Ok(PostgRestRequest {
        method: Method::POST,
        path: format!("{}/{table}", config.base_url),
        query_params: qp,
        headers,
        body: Some(data.clone()),
    })
}

fn build_rpc_request(
    req: &DbRequest,
    config: &PostgRestConfig,
) -> Result<PostgRestRequest, String> {
    let func = req
        .function_name
        .as_deref()
        .ok_or_else(|| "Action 'rpc' requires 'function_name' field".to_string())?;

    // Validate function name same as table name
    validate_table_name(func)
        .map_err(|e| e.replace("table", "function").replace("Table", "Function"))?;

    let headers = base_headers(req, config);
    let body = req.params.clone().unwrap_or(Value::Object(Default::default()));

    Ok(PostgRestRequest {
        method: Method::POST,
        path: format!("{}/rpc/{func}", config.base_url),
        query_params: Vec::new(),
        headers,
        body: Some(body),
    })
}

fn build_list_tables_request(config: &PostgRestConfig) -> Result<PostgRestRequest, String> {
    let mut headers = HeaderMap::new();
    if let Some(ref key) = config.anon_key {
        headers.insert("Authorization", format!("Bearer {key}").parse().unwrap());
    }

    Ok(PostgRestRequest {
        method: Method::GET,
        path: format!("{}/", config.base_url),
        query_params: Vec::new(),
        headers,
        body: None,
    })
}

fn build_describe_request(
    req: &DbRequest,
    config: &PostgRestConfig,
) -> Result<PostgRestRequest, String> {
    // Validate table exists in request (used for post-processing, not in URL)
    let _table = req.table.as_deref().unwrap();
    let headers = base_headers(req, config);

    // GET root endpoint returns the full OpenAPI spec; we extract the table
    // definition from `definitions.{table}` in execute_db post-processing.
    Ok(PostgRestRequest {
        method: Method::GET,
        path: config.base_url.clone(),
        query_params: Vec::new(),
        headers,
        body: None,
    })
}

// ---------------------------------------------------------------------------
// Response Normalizer
// ---------------------------------------------------------------------------

pub async fn normalize_response(
    result: Result<reqwest::Response, reqwest::Error>,
    action: &str,
    table: Option<&str>,
    start: Instant,
) -> DbResponse {
    let response = match result {
        Ok(r) => r,
        Err(e) => {
            let msg = if e.is_timeout() {
                format!("PostgREST request timed out: {e}")
            } else if e.is_connect() {
                format!("PostgREST connection failed: {e}")
            } else {
                format!("PostgREST request error: {e}")
            };
            return DbResponse::err(msg, action, table, start);
        }
    };

    let status = response.status();
    let content_range = response
        .headers()
        .get("content-range")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Parse count from Content-Range: 0-9/42 or */42
    let range_count = content_range.as_deref().and_then(parse_content_range_count);

    if status == StatusCode::NO_CONTENT {
        return DbResponse::ok(None, range_count, Some(0), action, table, start);
    }

    let body_text = match response.text().await {
        Ok(t) => t,
        Err(e) => {
            return DbResponse::err(
                format!("Failed to read PostgREST response body: {e}"),
                action,
                table,
                start,
            );
        }
    };

    if status.is_success() {
        let data: Value = match serde_json::from_str(&body_text) {
            Ok(v) => v,
            Err(_) => {
                // Some endpoints return non-JSON (e.g., OpenAPI spec as text)
                Value::String(body_text)
            }
        };

        let affected = data.as_array().map(|a| a.len());
        let count = range_count.or_else(|| affected.map(|a| a as i64));

        DbResponse::ok(Some(data), count, affected, action, table, start)
    } else {
        // Error response
        let error_msg = match serde_json::from_str::<Value>(&body_text) {
            Ok(err_json) => {
                let mut parts = Vec::new();
                if let Some(msg) = err_json.get("message").and_then(|v| v.as_str()) {
                    parts.push(msg.to_string());
                }
                if let Some(details) = err_json.get("details").and_then(|v| v.as_str()) {
                    if !details.is_empty() {
                        parts.push(format!("Details: {details}"));
                    }
                }
                if let Some(hint) = err_json.get("hint").and_then(|v| v.as_str()) {
                    if !hint.is_empty() {
                        parts.push(format!("Hint: {hint}"));
                    }
                }
                if parts.is_empty() {
                    format!("PostgREST error {status}: {body_text}")
                } else {
                    parts.join(". ")
                }
            }
            Err(_) => format!("PostgREST error {status}: {body_text}"),
        };

        DbResponse::err(error_msg, action, table, start)
    }
}

fn parse_content_range_count(header: &str) -> Option<i64> {
    // Format: "0-9/42" or "*/42"
    header
        .split('/')
        .nth(1)
        .and_then(|s| s.trim().parse::<i64>().ok())
}

// ---------------------------------------------------------------------------
// Execute (main entry point)
// ---------------------------------------------------------------------------

/// Execute a database action via PostgREST.
/// This is called by McpServer / ProtocolHandler.
pub async fn execute_db(
    client: &Client,
    config: &PostgRestConfig,
    req: &DbRequest,
) -> DbResponse {
    let start = Instant::now();
    let action = req.action.to_lowercase();
    let table = req.table.as_deref();

    // Build the PostgREST HTTP request
    let pg_req = match build_request(req, config) {
        Ok(r) => r,
        Err(e) => return DbResponse::err(e, &action, table, start),
    };

    // Send HTTP request
    let mut builder = client.request(pg_req.method, &pg_req.path);

    for (key, val) in &pg_req.query_params {
        builder = builder.query(&[(key, val)]);
    }

    builder = builder.headers(pg_req.headers);

    if let Some(body) = pg_req.body {
        builder = builder.json(&body);
    }

    let result = builder.send().await;

    let mut response = normalize_response(result, &action, table, start).await;

    // Post-process: for "describe", extract the table definition from the
    // OpenAPI spec returned by the root endpoint.
    if action == "describe" {
        if let (true, Some(data), Some(tbl)) = (response.success, &response.data, table) {
            if let Some(definition) = data
                .get("definitions")
                .and_then(|d| d.get(tbl))
            {
                response.data = Some(definition.clone());
            } else {
                response = DbResponse::err(
                    format!("Table '{tbl}' not found in PostgREST schema"),
                    &action,
                    table,
                    start,
                );
            }
        }
    }

    response
}

// ---------------------------------------------------------------------------
// Lazy-initialized global client + config
// ---------------------------------------------------------------------------

use std::sync::OnceLock;

static DB_CLIENT: OnceLock<Client> = OnceLock::new();
static DB_CONFIG: OnceLock<PostgRestConfig> = OnceLock::new();

/// Get or initialize the shared reqwest client
pub fn get_client() -> &'static Client {
    DB_CLIENT.get_or_init(|| {
        let config = get_config();
        Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to build HTTP client")
    })
}

/// Get or initialize the shared PostgREST config
pub fn get_config() -> &'static PostgRestConfig {
    DB_CONFIG.get_or_init(PostgRestConfig::from_env)
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Config tests --

    #[test]
    fn test_config_defaults() {
        // Clear env vars for test
        std::env::remove_var("POSTGREST_URL");
        std::env::remove_var("POSTGREST_ANON_KEY");
        std::env::remove_var("POSTGREST_TIMEOUT");
        std::env::remove_var("DB_ALLOWED_TABLES");
        std::env::remove_var("DB_TABLE_PREFIX");

        let config = PostgRestConfig::from_env();
        assert_eq!(config.base_url, "http://localhost:3000");
        assert!(config.anon_key.is_none());
        assert_eq!(config.timeout_secs, 30);
        assert!(config.allowed_tables.is_none());
        assert!(config.table_prefix.is_none());
    }

    #[test]
    fn test_is_table_allowed_no_restrictions() {
        let config = PostgRestConfig {
            base_url: "http://localhost:3000".to_string(),
            anon_key: None,
            timeout_secs: 30,
            allowed_tables: None,
            table_prefix: None,
        };
        assert!(config.is_table_allowed("anything"));
        assert!(config.is_table_allowed("users"));
    }

    #[test]
    fn test_is_table_allowed_whitelist() {
        let config = PostgRestConfig {
            base_url: "http://localhost:3000".to_string(),
            anon_key: None,
            timeout_secs: 30,
            allowed_tables: Some(["users", "posts"].iter().map(|s| s.to_string()).collect()),
            table_prefix: None,
        };
        assert!(config.is_table_allowed("users"));
        assert!(config.is_table_allowed("posts"));
        assert!(!config.is_table_allowed("secrets"));
    }

    #[test]
    fn test_is_table_allowed_prefix() {
        let config = PostgRestConfig {
            base_url: "http://localhost:3000".to_string(),
            anon_key: None,
            timeout_secs: 30,
            allowed_tables: None,
            table_prefix: Some("bdtv_".to_string()),
        };
        assert!(config.is_table_allowed("bdtv_users"));
        assert!(config.is_table_allowed("bdtv_credit_wallets"));
        assert!(!config.is_table_allowed("other_table"));
    }

    #[test]
    fn test_is_table_allowed_whitelist_and_prefix() {
        let config = PostgRestConfig {
            base_url: "http://localhost:3000".to_string(),
            anon_key: None,
            timeout_secs: 30,
            allowed_tables: Some(["extra_table"].iter().map(|s| s.to_string()).collect()),
            table_prefix: Some("app_".to_string()),
        };
        assert!(config.is_table_allowed("app_users")); // prefix match
        assert!(config.is_table_allowed("extra_table")); // whitelist match
        assert!(!config.is_table_allowed("other")); // neither
    }

    // -- Table name validation --

    #[test]
    fn test_validate_table_name_valid() {
        assert!(validate_table_name("users").is_ok());
        assert!(validate_table_name("bdtv_credit_wallets").is_ok());
        assert!(validate_table_name("_private").is_ok());
        assert!(validate_table_name("Table1").is_ok());
    }

    #[test]
    fn test_validate_table_name_invalid() {
        assert!(validate_table_name("").is_err());
        assert!(validate_table_name("1table").is_err());
        assert!(validate_table_name("drop;--").is_err());
        assert!(validate_table_name("table name").is_err());
        assert!(validate_table_name("../etc/passwd").is_err());
    }

    // -- Filter translation --

    #[test]
    fn test_filter_eq() {
        let filters = serde_json::json!({ "name": { "eq": "John" } });
        let result = translate_filters(&filters).unwrap();
        assert_eq!(result, vec![("name".to_string(), "eq.John".to_string())]);
    }

    #[test]
    fn test_filter_neq() {
        let filters = serde_json::json!({ "status": { "neq": "deleted" } });
        let result = translate_filters(&filters).unwrap();
        assert_eq!(
            result,
            vec![("status".to_string(), "neq.deleted".to_string())]
        );
    }

    #[test]
    fn test_filter_gt_lt() {
        let filters = serde_json::json!({ "age": { "gt": 18 } });
        let result = translate_filters(&filters).unwrap();
        assert_eq!(result, vec![("age".to_string(), "gt.18".to_string())]);
    }

    #[test]
    fn test_filter_in() {
        let filters = serde_json::json!({ "id": { "in": [1, 2, 3] } });
        let result = translate_filters(&filters).unwrap();
        assert_eq!(
            result,
            vec![("id".to_string(), "in.(1,2,3)".to_string())]
        );
    }

    #[test]
    fn test_filter_like_percent_to_star() {
        let filters = serde_json::json!({ "name": { "like": "%john%" } });
        let result = translate_filters(&filters).unwrap();
        assert_eq!(
            result,
            vec![("name".to_string(), "like.*john*".to_string())]
        );
    }

    #[test]
    fn test_filter_is_null() {
        let filters = serde_json::json!({ "deleted_at": { "is": null } });
        let result = translate_filters(&filters).unwrap();
        assert_eq!(
            result,
            vec![("deleted_at".to_string(), "is.null".to_string())]
        );
    }

    #[test]
    fn test_filter_is_true() {
        let filters = serde_json::json!({ "active": { "is": true } });
        let result = translate_filters(&filters).unwrap();
        assert_eq!(
            result,
            vec![("active".to_string(), "is.true".to_string())]
        );
    }

    #[test]
    fn test_filter_null_shorthand() {
        let filters = serde_json::json!({ "deleted_at": null });
        let result = translate_filters(&filters).unwrap();
        assert_eq!(
            result,
            vec![("deleted_at".to_string(), "is.null".to_string())]
        );
    }

    #[test]
    fn test_filter_simple_equality() {
        let filters = serde_json::json!({ "status": "active" });
        let result = translate_filters(&filters).unwrap();
        assert_eq!(
            result,
            vec![("status".to_string(), "eq.active".to_string())]
        );
    }

    #[test]
    fn test_filter_contains() {
        let filters = serde_json::json!({ "tags": { "contains": ["a", "b"] } });
        let result = translate_filters(&filters).unwrap();
        assert_eq!(
            result,
            vec![("tags".to_string(), "cs.{a,b}".to_string())]
        );
    }

    #[test]
    fn test_filter_contained_by() {
        let filters = serde_json::json!({ "tags": { "containedBy": ["a", "b", "c"] } });
        let result = translate_filters(&filters).unwrap();
        assert_eq!(
            result,
            vec![("tags".to_string(), "cd.{a,b,c}".to_string())]
        );
    }

    #[test]
    fn test_filter_overlaps() {
        let filters = serde_json::json!({ "tags": { "overlaps": ["x", "y"] } });
        let result = translate_filters(&filters).unwrap();
        assert_eq!(
            result,
            vec![("tags".to_string(), "ov.{x,y}".to_string())]
        );
    }

    #[test]
    fn test_filter_not() {
        let filters = serde_json::json!({ "status": { "not": "active" } });
        let result = translate_filters(&filters).unwrap();
        assert_eq!(
            result,
            vec![("status".to_string(), "not.eq.active".to_string())]
        );
    }

    #[test]
    fn test_filter_legacy_format() {
        let filters = serde_json::json!({ "eq": { "name": "John", "status": "active" } });
        let result = translate_filters(&filters).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&("name".to_string(), "eq.John".to_string())));
        assert!(result.contains(&("status".to_string(), "eq.active".to_string())));
    }

    #[test]
    fn test_filter_unknown_operator() {
        let filters = serde_json::json!({ "col": { "foobar": "val" } });
        let result = translate_filters(&filters);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown filter operator"));
    }

    // -- Order translation --

    #[test]
    fn test_order_single() {
        let order = serde_json::json!([{ "column": "created_at", "direction": "desc" }]);
        assert_eq!(translate_order(&order).unwrap(), "created_at.desc");
    }

    #[test]
    fn test_order_multiple() {
        let order = serde_json::json!([
            { "column": "name", "direction": "asc" },
            { "column": "id", "direction": "desc" }
        ]);
        assert_eq!(translate_order(&order).unwrap(), "name.asc,id.desc");
    }

    #[test]
    fn test_order_default_asc() {
        let order = serde_json::json!([{ "column": "name" }]);
        assert_eq!(translate_order(&order).unwrap(), "name.asc");
    }

    #[test]
    fn test_order_ascending_false() {
        let order = serde_json::json!([{ "column": "name", "ascending": false }]);
        assert_eq!(translate_order(&order).unwrap(), "name.desc");
    }

    #[test]
    fn test_order_string_passthrough() {
        let order = serde_json::json!("created_at.desc");
        assert_eq!(translate_order(&order).unwrap(), "created_at.desc");
    }

    // -- Select translation --

    #[test]
    fn test_select_string() {
        let select = serde_json::json!("id,name,email");
        assert_eq!(translate_select(&select).unwrap(), "id,name,email");
    }

    #[test]
    fn test_select_array() {
        let select = serde_json::json!(["id", "name", "email"]);
        assert_eq!(translate_select(&select).unwrap(), "id,name,email");
    }

    // -- Request builder --

    fn test_config() -> PostgRestConfig {
        PostgRestConfig {
            base_url: "http://localhost:3000".to_string(),
            anon_key: Some("test-key".to_string()),
            timeout_secs: 30,
            allowed_tables: Some(
                ["users", "posts", "test_mcp_db"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
            table_prefix: None,
        }
    }

    #[test]
    fn test_build_query() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "query",
            "table": "users",
            "select": "id,name",
            "filters": { "status": { "eq": "active" } },
            "order": [{ "column": "name", "direction": "asc" }],
            "limit": 10,
            "offset": 0
        }))
        .unwrap();

        let pg = build_request(&req, &config).unwrap();
        assert_eq!(pg.method, Method::GET);
        assert!(pg.path.ends_with("/users"));
        assert!(pg.query_params.contains(&("select".to_string(), "id,name".to_string())));
        assert!(pg.query_params.contains(&("status".to_string(), "eq.active".to_string())));
        assert!(pg.query_params.contains(&("order".to_string(), "name.asc".to_string())));
        assert!(pg.query_params.contains(&("limit".to_string(), "10".to_string())));
        assert!(pg.body.is_none());
    }

    #[test]
    fn test_build_insert() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "insert",
            "table": "users",
            "data": { "name": "John", "email": "john@example.com" }
        }))
        .unwrap();

        let pg = build_request(&req, &config).unwrap();
        assert_eq!(pg.method, Method::POST);
        assert!(pg.path.ends_with("/users"));
        assert!(pg.body.is_some());
        assert!(pg
            .headers
            .get("Prefer")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("return=representation"));
    }

    #[test]
    fn test_build_update() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "update",
            "table": "users",
            "data": { "name": "Jane" },
            "filters": { "id": { "eq": 1 } }
        }))
        .unwrap();

        let pg = build_request(&req, &config).unwrap();
        assert_eq!(pg.method, Method::PATCH);
        assert!(pg.query_params.contains(&("id".to_string(), "eq.1".to_string())));
        assert!(pg.body.is_some());
    }

    #[test]
    fn test_build_update_no_filters_rejected() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "update",
            "table": "users",
            "data": { "name": "Jane" }
        }))
        .unwrap();

        let result = build_request(&req, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("filters"));
    }

    #[test]
    fn test_build_delete() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "delete",
            "table": "users",
            "filters": { "id": { "eq": 1 } }
        }))
        .unwrap();

        let pg = build_request(&req, &config).unwrap();
        assert_eq!(pg.method, Method::DELETE);
        assert!(pg.query_params.contains(&("id".to_string(), "eq.1".to_string())));
        assert!(pg.body.is_none());
    }

    #[test]
    fn test_build_delete_no_filters_rejected() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "delete",
            "table": "users"
        }))
        .unwrap();

        let result = build_request(&req, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("filters"));
    }

    #[test]
    fn test_build_upsert() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "upsert",
            "table": "users",
            "data": { "id": 1, "name": "John" },
            "conflict": "id"
        }))
        .unwrap();

        let pg = build_request(&req, &config).unwrap();
        assert_eq!(pg.method, Method::POST);
        assert!(pg.query_params.contains(&("on_conflict".to_string(), "id".to_string())));
        let prefer = pg.headers.get("Prefer").unwrap().to_str().unwrap();
        assert!(prefer.contains("merge-duplicates"));
    }

    #[test]
    fn test_build_rpc() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "rpc",
            "function_name": "my_func",
            "params": { "x": 1 }
        }))
        .unwrap();

        let pg = build_request(&req, &config).unwrap();
        assert_eq!(pg.method, Method::POST);
        assert!(pg.path.contains("/rpc/my_func"));
        assert!(pg.body.is_some());
    }

    #[test]
    fn test_build_rpc_missing_function() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "rpc"
        }))
        .unwrap();

        let result = build_request(&req, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("function_name"));
    }

    #[test]
    fn test_build_list_tables() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "list_tables"
        }))
        .unwrap();

        let pg = build_request(&req, &config).unwrap();
        assert_eq!(pg.method, Method::GET);
        assert!(pg.path.ends_with('/'));
    }

    #[test]
    fn test_build_describe() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "describe",
            "table": "users"
        }))
        .unwrap();

        let pg = build_request(&req, &config).unwrap();
        // describe now GETs the root OpenAPI spec; table extraction is post-processed in execute_db
        assert_eq!(pg.method, Method::GET);
        assert_eq!(pg.path, config.base_url);
    }

    #[test]
    fn test_build_raw_sql_rejected() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "raw_sql",
            "sql": "SELECT 1"
        }))
        .unwrap();

        let result = build_request(&req, &config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("not supported"));
        assert!(err.contains("rpc"));
    }

    #[test]
    fn test_build_unknown_action() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "foobar"
        }))
        .unwrap();

        let result = build_request(&req, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown action"));
    }

    #[test]
    fn test_build_table_not_allowed() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "query",
            "table": "secrets"
        }))
        .unwrap();

        let result = build_request(&req, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not in the allowed"));
    }

    #[test]
    fn test_build_invalid_table_name() {
        let config = PostgRestConfig {
            base_url: "http://localhost:3000".to_string(),
            anon_key: None,
            timeout_secs: 30,
            allowed_tables: None,
            table_prefix: None,
        };
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "query",
            "table": "../hack"
        }))
        .unwrap();

        let result = build_request(&req, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid table name"));
    }

    #[test]
    fn test_build_missing_table() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "query"
        }))
        .unwrap();

        let result = build_request(&req, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires 'table'"));
    }

    #[test]
    fn test_build_insert_missing_data() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "insert",
            "table": "users"
        }))
        .unwrap();

        let result = build_request(&req, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires 'data'"));
    }

    #[test]
    fn test_build_with_token() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "query",
            "table": "users",
            "token": "my-jwt-token"
        }))
        .unwrap();

        let pg = build_request(&req, &config).unwrap();
        let auth = pg.headers.get("Authorization").unwrap().to_str().unwrap();
        assert_eq!(auth, "Bearer my-jwt-token");
    }

    #[test]
    fn test_build_with_anon_key_fallback() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "query",
            "table": "users"
        }))
        .unwrap();

        let pg = build_request(&req, &config).unwrap();
        let auth = pg.headers.get("Authorization").unwrap().to_str().unwrap();
        assert_eq!(auth, "Bearer test-key");
    }

    #[test]
    fn test_build_with_options_single() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "query",
            "table": "users",
            "filters": { "id": { "eq": 1 } },
            "options": { "single": true }
        }))
        .unwrap();

        let pg = build_request(&req, &config).unwrap();
        let accept = pg.headers.get("Accept").unwrap().to_str().unwrap();
        assert_eq!(accept, "application/vnd.pgrst.object+json");
    }

    #[test]
    fn test_build_with_options_count() {
        let config = test_config();
        let req = serde_json::from_value::<DbRequest>(serde_json::json!({
            "action": "query",
            "table": "users",
            "options": { "count": "exact" }
        }))
        .unwrap();

        let pg = build_request(&req, &config).unwrap();
        let prefer = pg.headers.get("Prefer").unwrap().to_str().unwrap();
        assert!(prefer.contains("count=exact"));
    }

    // -- Content-Range parsing --

    #[test]
    fn test_parse_content_range_count() {
        assert_eq!(parse_content_range_count("0-9/42"), Some(42));
        assert_eq!(parse_content_range_count("*/100"), Some(100));
        assert_eq!(parse_content_range_count("0-0/1"), Some(1));
        assert_eq!(parse_content_range_count("invalid"), None);
        assert_eq!(parse_content_range_count("0-9/*"), None);
    }

    // -- DbRequest deserialization --

    #[test]
    fn test_request_minimal() {
        let req: DbRequest =
            serde_json::from_value(serde_json::json!({ "action": "list_tables" })).unwrap();
        assert_eq!(req.action, "list_tables");
        assert!(req.table.is_none());
        assert!(req.filters.is_none());
    }

    #[test]
    fn test_request_full() {
        let req: DbRequest = serde_json::from_value(serde_json::json!({
            "action": "query",
            "table": "users",
            "select": "id,name",
            "filters": { "status": "active" },
            "order": [{ "column": "name" }],
            "limit": 10,
            "offset": 5,
            "options": { "count": "exact", "single": false }
        }))
        .unwrap();
        assert_eq!(req.action, "query");
        assert_eq!(req.table.as_deref(), Some("users"));
        assert_eq!(req.limit, Some(10));
        assert_eq!(req.offset, Some(5));
    }

    #[test]
    fn test_request_function_alias() {
        let req: DbRequest = serde_json::from_value(serde_json::json!({
            "action": "rpc",
            "function": "my_func"
        }))
        .unwrap();
        assert_eq!(req.function_name.as_deref(), Some("my_func"));
    }

    // -- DbResponse serialization --

    #[test]
    fn test_response_ok_serialization() {
        let start = Instant::now();
        let resp = DbResponse::ok(
            Some(serde_json::json!([{"id": 1}])),
            Some(1),
            Some(1),
            "query",
            Some("users"),
            start,
        );
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["success"], true);
        assert!(json.get("error").is_none()); // skip_serializing_if
        assert_eq!(json["count"], 1);
        assert_eq!(json["metadata"]["action"], "query");
        assert_eq!(json["metadata"]["table"], "users");
    }

    #[test]
    fn test_response_err_serialization() {
        let start = Instant::now();
        let resp = DbResponse::err("something failed", "insert", Some("users"), start);
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["error"], "something failed");
        assert!(json.get("data").is_none());
        assert!(json.get("count").is_none());
    }
}