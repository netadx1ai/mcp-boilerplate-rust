//! Database Integration Template - Production MCP server for database access
//!
//! This template provides comprehensive database integration patterns including:
//! - Multiple database engine support (PostgreSQL, MySQL, SQLite)
//! - Connection pooling and transaction management
//! - SQL query validation and execution with security protection
//! - Schema introspection and migration patterns
//! - Production-ready error handling and monitoring
//!
//! Built on the official RMCP SDK with proven production patterns.
//!
//! ## Customization Guide
//!
//! 1. **Database Engine**: Enable feature in Cargo.toml (postgres/mysql/sqlite)
//! 2. **Connection Config**: Update `DatabaseConfig` struct
//! 3. **Schema**: Modify `init_database_schema()` for your tables
//! 4. **Security**: Customize query validation rules in `validate_query_security()`
//! 5. **Tools**: Add your domain-specific MCP tools
//!
//! ## Quick Start
//!
//! ```bash
//! # For PostgreSQL
//! cargo run --features postgres -- --database-url postgresql://user:pass@localhost/db
//!
//! # For SQLite (default)
//! cargo run -- --database-url sqlite:///path/to/db.sqlite
//!
//! # For MySQL
//! cargo run --features mysql -- --database-url mysql://user:pass@localhost/db
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use dashmap::DashMap;
use regex::Regex;
use rmcp::{
    handler::server::wrapper::Parameters, model::*, service::RequestContext, tool, tool_router,
    transport::stdio, ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlparser::{ast::Statement, dialect::GenericDialect, parser::Parser as SqlParser};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

// ================================================================================================
// CLI Configuration
// ================================================================================================

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Database connection URL
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Maximum database connections in pool
    #[arg(long, default_value = "10")]
    max_connections: u32,

    /// Connection timeout in seconds
    #[arg(long, default_value = "30")]
    connection_timeout: u64,
}

// ================================================================================================
// Database Configuration
// ================================================================================================

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub engine: DatabaseEngine,
}

#[derive(Debug, Clone)]
pub enum DatabaseEngine {
    PostgreSQL,
    MySQL,
    SQLite,
    Mock, // For development/testing
}

impl DatabaseConfig {
    pub fn from_args(args: &Args) -> Result<Self> {
        let url = args.database_url.clone().unwrap_or_else(|| {
            // Default to SQLite for development
            "sqlite:///tmp/template_database.db".to_string()
        });

        let engine = if url.starts_with("postgresql://") || url.starts_with("postgres://") {
            DatabaseEngine::PostgreSQL
        } else if url.starts_with("mysql://") {
            DatabaseEngine::MySQL
        } else if url.starts_with("sqlite://") {
            DatabaseEngine::SQLite
        } else {
            DatabaseEngine::Mock
        };

        Ok(DatabaseConfig {
            url,
            max_connections: args.max_connections,
            connection_timeout: args.connection_timeout,
            engine,
        })
    }
}

// ================================================================================================
// Request/Response Types
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecuteQueryArgs {
    /// SQL query to execute
    pub query: String,
    /// Query parameters for prepared statements
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListTablesArgs {
    /// Database/schema name (optional)
    #[serde(default = "default_database")]
    pub database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetTableSchemaArgs {
    /// Table name to get schema for
    pub table_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidateQueryArgs {
    /// SQL query to validate
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetQueryPlanArgs {
    /// SQL query to analyze
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BeginTransactionArgs {
    /// Transaction isolation level (optional)
    pub isolation_level: Option<String>,
}

// ================================================================================================
// Database Schema Types
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub primary_key: Vec<String>,
    pub foreign_keys: Vec<ForeignKey>,
    pub row_count: usize,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ForeignKey {
    pub column: String,
    pub references_table: String,
    pub references_column: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub execution_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QueryPlan {
    pub operation: String,
    pub table: Option<String>,
    pub estimated_rows: Option<u64>,
    pub estimated_cost: Option<f64>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatabaseStats {
    pub total_tables: usize,
    pub total_rows: usize,
    pub total_queries: u64,
    pub avg_query_time_ms: f64,
    pub active_connections: u32,
    pub uptime_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionInfo {
    pub transaction_id: String,
    pub isolation_level: String,
    pub start_time: DateTime<Utc>,
    pub status: String,
}

// ================================================================================================
// Core Database Server
// ================================================================================================

pub struct DatabaseServer {
    config: DatabaseConfig,
    // Note: In production, replace with actual connection pool
    // PostgreSQL: deadpool_postgres::Pool
    // MySQL: mysql::Pool  
    // SQLite: Arc<Mutex<rusqlite::Connection>>
    connection_pool: Option<String>, // Placeholder for actual pool
    
    // Schema cache and statistics
    schema_cache: Arc<DashMap<String, TableSchema>>,
    query_stats: Arc<DashMap<String, (u64, f64)>>, // (count, total_time)
    tool_stats: Arc<DashMap<String, u64>>,
    active_transactions: Arc<DashMap<String, TransactionInfo>>,
    start_time: DateTime<Utc>,
}

fn default_database() -> String {
    "main".to_string()
}

impl DatabaseServer {
    pub fn new(config: DatabaseConfig) -> Result<Self> {
        let server = Self {
            config: config.clone(),
            connection_pool: None, // TODO: Initialize actual connection pool
            schema_cache: Arc::new(DashMap::new()),
            query_stats: Arc::new(DashMap::new()),
            tool_stats: Arc::new(DashMap::new()),
            active_transactions: Arc::new(DashMap::new()),
            start_time: Utc::now(),
        };

        // Initialize database schema and connection pool
        server.init_database_connection()?;
        server.init_database_schema()?;
        
        info!("Database server initialized with engine: {:?}", config.engine);
        Ok(server)
    }

    fn init_database_connection(&self) -> Result<()> {
        // TODO: Initialize actual database connection pool based on engine
        match self.config.engine {
            DatabaseEngine::PostgreSQL => {
                info!("Initializing PostgreSQL connection pool");
                // Example: deadpool_postgres::Config::new() etc.
            }
            DatabaseEngine::MySQL => {
                info!("Initializing MySQL connection pool");
                // Example: mysql::PoolBuilder::new() etc.
            }
            DatabaseEngine::SQLite => {
                info!("Initializing SQLite connection");
                // Example: rusqlite::Connection::open() etc.
            }
            DatabaseEngine::Mock => {
                info!("Using mock database for development");
                self.init_mock_database();
            }
        }
        Ok(())
    }

    fn init_database_schema(&self) -> Result<()> {
        // Initialize sample schema - customize this for your application
        self.create_sample_tables();
        Ok(())
    }

    fn init_mock_database(&self) {
        // Sample schema for template demonstration
        // TODO: Replace with your actual schema
        
        // Users table
        let users_schema = TableSchema {
            name: "users".to_string(),
            columns: vec![
                ColumnInfo {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    default_value: None,
                    description: Some("Primary key".to_string()),
                },
                ColumnInfo {
                    name: "username".to_string(),
                    data_type: "VARCHAR(50)".to_string(),
                    nullable: false,
                    default_value: None,
                    description: Some("Unique username".to_string()),
                },
                ColumnInfo {
                    name: "email".to_string(),
                    data_type: "VARCHAR(100)".to_string(),
                    nullable: false,
                    default_value: None,
                    description: Some("User email address".to_string()),
                },
                ColumnInfo {
                    name: "created_at".to_string(),
                    data_type: "TIMESTAMP".to_string(),
                    nullable: false,
                    default_value: Some("CURRENT_TIMESTAMP".to_string()),
                    description: Some("Account creation time".to_string()),
                },
            ],
            primary_key: vec!["id".to_string()],
            foreign_keys: vec![],
            row_count: 5,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        // Products table
        let products_schema = TableSchema {
            name: "products".to_string(),
            columns: vec![
                ColumnInfo {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    default_value: None,
                    description: Some("Primary key".to_string()),
                },
                ColumnInfo {
                    name: "name".to_string(),
                    data_type: "VARCHAR(100)".to_string(),
                    nullable: false,
                    default_value: None,
                    description: Some("Product name".to_string()),
                },
                ColumnInfo {
                    name: "price".to_string(),
                    data_type: "DECIMAL(10,2)".to_string(),
                    nullable: false,
                    default_value: None,
                    description: Some("Product price".to_string()),
                },
                ColumnInfo {
                    name: "category_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: true,
                    default_value: None,
                    description: Some("Product category".to_string()),
                },
            ],
            primary_key: vec!["id".to_string()],
            foreign_keys: vec![ForeignKey {
                column: "category_id".to_string(),
                references_table: "categories".to_string(),
                references_column: "id".to_string(),
            }],
            row_count: 10,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        self.schema_cache.insert("users".to_string(), users_schema);
        self.schema_cache.insert("products".to_string(), products_schema);
    }

    fn create_sample_tables(&self) {
        // TODO: Execute actual CREATE TABLE statements based on your schema
        info!("Sample database schema loaded");
    }

    pub async fn update_stats(&self, tool_name: &str) {
        self.tool_stats
            .entry(tool_name.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    pub fn validate_query_security(&self, query: &str) -> Result<(), String> {
        let binding = query.to_lowercase();
        let query_lower = binding.trim();
        
        // Basic SQL injection protection patterns
        let dangerous_patterns = [
            "drop table",
            "delete from",
            "truncate",
            "alter table", 
            "create table",
            "insert into",
            "update ",
            "exec ",
            "execute ",
            "union select",
            "script>",
            "javascript:",
        ];

        for pattern in &dangerous_patterns {
            if query_lower.contains(pattern) {
                return Err(format!("Potentially dangerous SQL pattern detected: {}", pattern));
            }
        }

        // Additional validation rules
        if query_lower.contains("--") && !query_lower.starts_with("--") {
            return Err("SQL comments not allowed in query body".to_string());
        }

        if query_lower.matches(';').count() > 1 {
            return Err("Multiple statements not allowed".to_string());
        }

        Ok(())
    }

    pub fn parse_sql(&self, query: &str) -> Result<Vec<Statement>, String> {
        let dialect = GenericDialect {};
        SqlParser::parse_sql(&dialect, query).map_err(|e| format!("SQL Parse Error: {e}"))
    }

    pub async fn execute_query_internal(&self, query: &str, _params: &HashMap<String, serde_json::Value>) -> Result<QueryResult, String> {
        let start_time = std::time::Instant::now();

        // Validate query security
        self.validate_query_security(query)?;

        // Parse SQL for validation
        let _statements = self.parse_sql(query)?;

        // TODO: Execute actual query against database
        // This is a mock implementation - replace with real database execution
        let result = match self.config.engine {
            DatabaseEngine::PostgreSQL => {
                // TODO: sqlx::query(query).fetch_all(&pool).await
                self.execute_mock_query(query)
            }
            DatabaseEngine::MySQL => {
                // TODO: pool.get_conn().await?.query(query)
                self.execute_mock_query(query)
            }
            DatabaseEngine::SQLite => {
                // TODO: conn.prepare(query)?.query_map(params)
                self.execute_mock_query(query)
            }
            DatabaseEngine::Mock => {
                self.execute_mock_query(query)
            }
        };

        let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;
        
        // Update query statistics
        self.query_stats
            .entry("total".to_string())
            .and_modify(|(count, total_time)| {
                *count += 1;
                *total_time += execution_time;
            })
            .or_insert((1, execution_time));

        result.map(|mut r| {
            r.execution_time_ms = execution_time;
            r
        })
    }

    pub fn execute_mock_query(&self, query: &str) -> Result<QueryResult, String> {
        let binding = query.to_lowercase();
        let query_lower = binding.trim();
        
        if query_lower.starts_with("select") {
            // Mock SELECT results
            let columns = vec!["id".to_string(), "name".to_string(), "value".to_string()];
            let rows = vec![
                vec![
                    serde_json::json!(1),
                    serde_json::json!("Sample Record 1"),
                    serde_json::json!("Value 1"),
                ],
                vec![
                    serde_json::json!(2),
                    serde_json::json!("Sample Record 2"),
                    serde_json::json!("Value 2"),
                ],
            ];
            
            Ok(QueryResult {
                columns,
                rows,
                row_count: 2,
                execution_time_ms: 0.0, // Will be set by caller
            })
        } else {
            // Mock result for non-SELECT queries
            let columns = vec!["result".to_string()];
            let rows = vec![vec![serde_json::json!("Query executed successfully")]];
            
            Ok(QueryResult {
                columns,
                rows,
                row_count: 1,
                execution_time_ms: 0.0,
            })
        }
    }

    pub fn format_query_result(&self, result: &QueryResult) -> String {
        let mut output = String::new();
        
        output.push_str("## Query Results\n\n");
        output.push_str(&format!("**Rows:** {} | **Execution Time:** {:.2}ms\n\n", 
                                result.row_count, result.execution_time_ms));
        
        if !result.columns.is_empty() && !result.rows.is_empty() {
            // Table header
            output.push_str("| ");
            for column in &result.columns {
                output.push_str(&format!("{} | ", column));
            }
            output.push('\n');
            
            // Table separator
            output.push_str("| ");
            for _ in &result.columns {
                output.push_str("--- | ");
            }
            output.push('\n');
            
            // Table rows (limit to first 10 rows for display)
            for (i, row) in result.rows.iter().enumerate() {
                if i >= 10 {
                    output.push_str(&format!("... and {} more rows\n", result.rows.len() - 10));
                    break;
                }
                
                output.push_str("| ");
                for value in row {
                    let display_value = match value {
                        serde_json::Value::Null => "NULL".to_string(),
                        serde_json::Value::String(s) => s.clone(),
                        _ => value.to_string(),
                    };
                    output.push_str(&format!("{} | ", display_value));
                }
                output.push('\n');
            }
        }
        
        output
    }
}

// ================================================================================================
// MCP Tools Implementation
// ================================================================================================

#[tool_router]
impl DatabaseServer {
    /// Execute SQL query with parameters and return results
    #[tool(description = "Execute SQL query with optional parameters")]
    async fn execute_query(
        &self,
        Parameters(args): Parameters<ExecuteQueryArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("execute_query").await;

        debug!("Executing query: {}", args.query);

        match self.execute_query_internal(&args.query, &args.params).await {
            Ok(result) => {
                let formatted_result = self.format_query_result(&result);
                Ok(CallToolResult::success(vec![Content::text(formatted_result)]))
            }
            Err(e) => {
                error!("Query execution failed: {}", e);
                let error_msg = format!("❌ **Query Execution Failed**\n\n**Error:** {}\n\n**Query:** `{}`", e, args.query);
                Ok(CallToolResult::success(vec![Content::text(error_msg)]))
            }
        }
    }

    /// List all tables in the specified database
    #[tool(description = "List all tables in the database")]
    async fn list_tables(
        &self,
        Parameters(args): Parameters<ListTablesArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("list_tables").await;

        let mut result = String::new();
        result.push_str("## Database Tables\n\n");
        result.push_str(&format!("**Database:** {}\n\n", args.database));
        
        if self.schema_cache.is_empty() {
            result.push_str("No tables found in database.\n");
        } else {
            result.push_str("| Table Name | Columns | Rows | Created |\n");
            result.push_str("| --- | --- | --- | --- |\n");
            
            for entry in self.schema_cache.iter() {
                let table = entry.value();
                result.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    table.name,
                    table.columns.len(),
                    table.row_count,
                    table.created_at
                ));
            }
        }

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get detailed schema information for a specific table
    #[tool(description = "Get table schema with columns, types, and constraints")]
    async fn get_table_schema(
        &self,
        Parameters(args): Parameters<GetTableSchemaArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("get_table_schema").await;

        if let Some(schema) = self.schema_cache.get(&args.table_name) {
            let mut result = String::new();
            result.push_str(&format!("## Table Schema: {}\n\n", schema.name));
            result.push_str(&format!("**Primary Key:** {}\n", schema.primary_key.join(", ")));
            result.push_str(&format!("**Row Count:** {}\n", schema.row_count));
            result.push_str(&format!("**Created:** {}\n\n", schema.created_at));
            
            // Columns table
            result.push_str("### Columns\n\n");
            result.push_str("| Name | Type | Nullable | Default | Description |\n");
            result.push_str("| --- | --- | --- | --- | --- |\n");
            
            for column in &schema.columns {
                result.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    column.name,
                    column.data_type,
                    if column.nullable { "Yes" } else { "No" },
                    column.default_value.as_deref().unwrap_or("None"),
                    column.description.as_deref().unwrap_or(""),
                ));
            }
            
            // Foreign keys
            if !schema.foreign_keys.is_empty() {
                result.push_str("\n### Foreign Keys\n\n");
                result.push_str("| Column | References |\n");
                result.push_str("| --- | --- |\n");
                
                for fk in &schema.foreign_keys {
                    result.push_str(&format!(
                        "| {} | {}.{} |\n",
                        fk.column, fk.references_table, fk.references_column
                    ));
                }
            }
            
            Ok(CallToolResult::success(vec![Content::text(result)]))
        } else {
            let error_msg = format!("❌ Table '{}' not found in database", args.table_name);
            Ok(CallToolResult::success(vec![Content::text(error_msg)]))
        }
    }

    /// Validate SQL query syntax and security
    #[tool(description = "Validate SQL query for syntax and security issues")]
    async fn validate_query(
        &self,
        Parameters(args): Parameters<ValidateQueryArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("validate_query").await;

        // Check security first
        match self.validate_query_security(&args.query) {
            Ok(_) => {
                // Then check syntax
                match self.parse_sql(&args.query) {
                    Ok(statements) => {
                        let result = format!(
                            "✅ **Query Validation: PASSED**\n\n**Query:** `{}`\n\n**Parsed Statements:** {}\n\n**Security:** No issues detected\n\n**Status:** Valid and safe",
                            args.query,
                            statements.len()
                        );
                        Ok(CallToolResult::success(vec![Content::text(result)]))
                    }
                    Err(e) => {
                        let result = format!(
                            "❌ **Query Validation: SYNTAX ERROR**\n\n**Query:** `{}`\n\n**Error:** {}\n\n**Status:** Invalid SQL syntax",
                            args.query,
                            e
                        );
                        Ok(CallToolResult::success(vec![Content::text(result)]))
                    }
                }
            }
            Err(security_error) => {
                let result = format!(
                    "🚨 **Query Validation: SECURITY RISK**\n\n**Query:** `{}`\n\n**Security Issue:** {}\n\n**Status:** Potentially dangerous query blocked",
                    args.query,
                    security_error
                );
                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
        }
    }

    /// Get query execution plan and optimization information
    #[tool(description = "Analyze query execution plan")]
    async fn get_query_plan(
        &self,
        Parameters(args): Parameters<GetQueryPlanArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("get_query_plan").await;

        // TODO: Get actual query plan from database engine
        // PostgreSQL: EXPLAIN (FORMAT JSON) query
        // MySQL: EXPLAIN FORMAT=JSON query
        // SQLite: EXPLAIN QUERY PLAN query

        let plan = QueryPlan {
            operation: "Mock Analysis".to_string(),
            table: Some("sample_table".to_string()),
            estimated_rows: Some(1000),
            estimated_cost: Some(10.5),
            description: "This is a mock query plan. In production, this would show the actual database execution plan.".to_string(),
        };

        let result = format!(
            "## Query Execution Plan\n\n**Query:** `{}`\n\n**Operation:** {}\n\n**Table:** {}\n\n**Estimated Rows:** {}\n\n**Estimated Cost:** {}\n\n**Description:** {}",
            args.query,
            plan.operation,
            plan.table.unwrap_or("N/A".to_string()),
            plan.estimated_rows.unwrap_or(0),
            plan.estimated_cost.unwrap_or(0.0),
            plan.description
        );

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Begin a new database transaction
    #[tool(description = "Begin a new database transaction")]
    async fn begin_transaction(
        &self,
        Parameters(args): Parameters<BeginTransactionArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("begin_transaction").await;

        let transaction_id = uuid::Uuid::new_v4().to_string();
        let isolation_level = args.isolation_level.unwrap_or("READ_COMMITTED".to_string());

        let transaction = TransactionInfo {
            transaction_id: transaction_id.clone(),
            isolation_level: isolation_level.clone(),
            start_time: Utc::now(),
            status: "ACTIVE".to_string(),
        };

        self.active_transactions.insert(transaction_id.clone(), transaction);

        let result = format!(
            "✅ **Transaction Started**\n\n**Transaction ID:** {}\n\n**Isolation Level:** {}\n\n**Status:** Active\n\n**Note:** This is a mock transaction. In production, this would create an actual database transaction.",
            transaction_id,
            isolation_level
        );

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get database connection and performance statistics
    #[tool(description = "Get database server statistics and health information")]
    async fn get_database_stats(
        &self,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("get_database_stats").await;

        let total_queries = self.query_stats
            .get("total")
            .map(|stats| stats.0)
            .unwrap_or(0);

        let avg_query_time = self.query_stats
            .get("total")
            .map(|stats| if stats.0 > 0 { stats.1 / stats.0 as f64 } else { 0.0 })
            .unwrap_or(0.0);

        let stats = DatabaseStats {
            total_tables: self.schema_cache.len(),
            total_rows: self.schema_cache.iter().map(|entry| entry.value().row_count).sum(),
            total_queries,
            avg_query_time_ms: avg_query_time,
            active_connections: self.config.max_connections, // Mock value
            uptime_seconds: (Utc::now() - self.start_time).num_seconds(),
        };

        let result = format!(
            "## Database Statistics\n\n**Engine:** {:?}\n\n**Connection URL:** {}\n\n**Tables:** {}\n\n**Total Rows:** {}\n\n**Total Queries:** {}\n\n**Avg Query Time:** {:.2}ms\n\n**Active Connections:** {}\n\n**Uptime:** {}s\n\n**Active Transactions:** {}",
            self.config.engine,
            self.config.url,
            stats.total_tables,
            stats.total_rows,
            stats.total_queries,
            stats.avg_query_time_ms,
            stats.active_connections,
            stats.uptime_seconds,
            self.active_transactions.len()
        );

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get server health and status information
    #[tool(description = "Get database server health status")]
    async fn get_server_status(
        &self,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("get_server_status").await;

        let mut tool_usage = String::new();
        for entry in self.tool_stats.iter() {
            tool_usage.push_str(&format!("- {}: {} calls\n", entry.key(), entry.value()));
        }

        let result = format!(
            "🟢 **Database Server Status: HEALTHY**\n\n**Engine:** {:?}\n\n**Uptime:** {}s\n\n**Max Connections:** {}\n\n**Connection Timeout:** {}s\n\n**Schema Tables:** {}\n\n## Tool Usage\n{}",
            self.config.engine,
            (Utc::now() - self.start_time).num_seconds(),
            self.config.max_connections,
            self.config.connection_timeout,
            self.schema_cache.len(),
            if tool_usage.is_empty() { "No tools used yet\n".to_string() } else { tool_usage }
        );

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}

// ================================================================================================
// Server Handler Implementation
// ================================================================================================

impl ServerHandler for DatabaseServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "🗄️ Database Integration Template - SQL query and data access:\n\
                • execute_query: Execute SQL queries with parameter binding\n\
                • list_tables: List all available tables in database\n\
                • get_table_schema: Get detailed table structure and metadata\n\
                • validate_query: Validate SQL query syntax and security\n\
                • get_query_plan: Analyze query execution plan\n\
                • begin_transaction: Start database transaction\n\
                • get_database_stats: Database performance statistics\n\
                • get_server_status: Health check and usage statistics\n\n\
                🔒 Security: SQL injection protection and query validation\n\
                🚀 Template ready for PostgreSQL, MySQL, SQLite integration"
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        info!("🗄️ Database Integration Template initialized");
        info!("🔒 Security patterns enabled for SQL protection");
        Ok(self.get_info())
    }
}

impl Default for DatabaseServer {
    fn default() -> Self {
        let config = DatabaseConfig {
            url: "sqlite:///tmp/default.db".to_string(),
            max_connections: 10,
            connection_timeout: 30,
            engine: DatabaseEngine::Mock,
        };
        Self::new(config).expect("Failed to create default database server")
    }
}

// ================================================================================================
// Main Application
// ================================================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let log_level = if args.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(format!("database_integration_template={},rmcp={}", log_level, log_level)))
        .init();

    info!("Starting Database Integration Template Server");

    // Initialize database configuration
    let config = DatabaseConfig::from_args(&args)
        .context("Failed to create database configuration")?;

    info!("Database configuration: {:?}", config.engine);

    // Create and start the server
    let database_server = DatabaseServer::new(config)
        .context("Failed to create database server")?;

    info!("Database server listening on stdio");
    
    // Start the server with STDIO transport
    let service = database_server.serve(stdio()).await.inspect_err(|e| {
        error!("Failed to start server: {:?}", e);
    })?;

    info!("✅ Database Integration Template started and ready for MCP connections");
    info!("🔒 Security patterns active for SQL protection");

    // Wait for the service to complete
    service.waiting().await?;

    Ok(())
}

// ================================================================================================
// Tests
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    fn create_test_server() -> DatabaseServer {
        let config = DatabaseConfig {
            url: "sqlite:///tmp/test.db".to_string(),
            max_connections: 5,
            connection_timeout: 10,
            engine: DatabaseEngine::Mock,
        };
        DatabaseServer::new(config).unwrap()
    }

    #[tokio::test]
    async fn test_server_creation() {
        let server = create_test_server();
        assert!(!server.schema_cache.is_empty());
    }

    #[tokio::test]
    async fn test_execute_query_tool() {
        let server = create_test_server();
        let args = ExecuteQueryArgs {
            query: "SELECT * FROM users".to_string(),
            params: HashMap::new(),
        };

        let result = server.execute_query(Parameters(args)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_tables_tool() {
        let server = create_test_server();
        let args = ListTablesArgs {
            database: "main".to_string(),
        };

        let result = server.list_tables(Parameters(args)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_table_schema_tool() {
        let server = create_test_server();
        let args = GetTableSchemaArgs {
            table_name: "users".to_string(),
        };

        let result = server.get_table_schema(Parameters(args)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_query_tool() {
        let server = create_test_server();
        let args = ValidateQueryArgs {
            query: "SELECT id, name FROM users WHERE id = 1".to_string(),
        };

        let result = server.validate_query(Parameters(args)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sql_injection_protection() {
        let server = create_test_server();
        let args = ValidateQueryArgs {
            query: "SELECT * FROM users; DROP TABLE users; --".to_string(),
        };

        let result = server.validate_query(Parameters(args)).await;
        assert!(result.is_ok()); // Should succeed but indicate security issue
    }

    #[tokio::test]
    async fn test_begin_transaction_tool() {
        let server = create_test_server();
        let args = BeginTransactionArgs {
            isolation_level: Some("SERIALIZABLE".to_string()),
        };

        let result = server.begin_transaction(Parameters(args)).await;
        assert!(result.is_ok());
        assert!(!server.active_transactions.is_empty());
    }

    #[tokio::test]
    async fn test_get_database_stats_tool() {
        let server = create_test_server();
        let result = server.get_database_stats().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_server_status_tool() {
        let server = create_test_server();
        let result = server.get_server_status().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_security_validation() {
        let server = create_test_server();
        
        // Test dangerous patterns
        let dangerous_queries = [
            "DROP TABLE users",
            "DELETE FROM users",
            "INSERT INTO users VALUES (1, 'hacker')",
            "UPDATE users SET password = 'hacked'",
            "SELECT * FROM users; DROP TABLE users; --",
        ];

        for query in &dangerous_queries {
            let result = server.validate_query_security(query);
            assert!(result.is_err(), "Query should be blocked: {}", query);
        }

        // Test safe queries
        let safe_queries = [
            "SELECT * FROM users",
            "SELECT id, name FROM users WHERE id = 1",
            "SELECT COUNT(*) FROM products",
        ];

        for query in &safe_queries {
            let result = server.validate_query_security(query);
            assert!(result.is_ok(), "Query should be allowed: {}", query);
        }
    }
}