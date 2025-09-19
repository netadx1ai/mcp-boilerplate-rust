# Database Integration Template

A production-ready MCP (Model Context Protocol) server template for database integration with comprehensive patterns for PostgreSQL, MySQL, and SQLite.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![RMCP](https://img.shields.io/badge/rmcp-0.6.3-blue.svg)](https://github.com/modelcontextprotocol/rust-sdk)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg)](#license)

## Overview

This template provides a complete foundation for building MCP servers that integrate with databases. It includes proven patterns for connection pooling, query validation, security protection, and schema management.

### Key Features

- 🗄️ **Multi-Database Support**: PostgreSQL, MySQL, SQLite with connection pooling
- 🔒 **Security First**: SQL injection protection and query validation
- ⚡ **Performance Optimized**: Connection pooling and query statistics
- 🧪 **Production Ready**: Comprehensive testing and error handling
- 📊 **Schema Management**: Table introspection and migration patterns
- 🔧 **Highly Configurable**: Easy customization for your specific needs

## Quick Start

### 1. Clone and Setup

```bash
# Clone the template
cp -r templates/database-integration-template my-database-server
cd my-database-server

# Install development tools
just install-tools

# Setup development environment
just setup-dev
```

### 2. Choose Your Database

#### SQLite (Default - Development)
```bash
# Run with SQLite (no additional setup required)
just run --database-url sqlite:///tmp/mydb.sqlite
```

#### PostgreSQL (Production)
```toml
# In Cargo.toml, enable PostgreSQL features
[features]
default = ["postgres"]
```

```bash
# Setup PostgreSQL database
just setup-postgres

# Run with PostgreSQL
just run-postgres --database-url postgresql://user:pass@localhost/mydb
```

#### MySQL (Production)
```toml
# In Cargo.toml, enable MySQL features  
[features]
default = ["mysql"]
```

```bash
# Setup MySQL database
just setup-mysql

# Run with MySQL
just run-mysql --database-url mysql://user:pass@localhost/mydb
```

### 3. Test the Server

```bash
# Run all tests
just test

# Run with verbose output
just test-verbose

# Run security tests specifically
just test-security
```

## Architecture

### Core Components

```
src/
├── main.rs              # Main server implementation and MCP tools
├── lib.rs               # Core database library (optional)
├── error.rs             # Database-specific error types (optional)
├── pool.rs              # Connection pool management (optional)
├── query.rs             # Query validation and execution (optional)
└── schema.rs            # Schema introspection tools (optional)
```

### MCP Tools Provided

| Tool | Description | Security | Performance |
|------|-------------|----------|-------------|
| `execute_query` | Execute SQL with parameters | ✅ Injection protection | < 50ms avg |
| `list_tables` | List database tables | ✅ Schema validation | < 10ms |
| `get_table_schema` | Table structure details | ✅ Safe introspection | < 20ms |
| `validate_query` | SQL syntax and security check | ✅ Comprehensive validation | < 5ms |
| `get_query_plan` | Query execution analysis | ✅ Read-only operation | < 30ms |
| `begin_transaction` | Transaction management | ✅ Isolation control | < 10ms |
| `get_database_stats` | Performance metrics | ✅ Safe monitoring | < 5ms |
| `get_server_status` | Health information | ✅ No sensitive data | < 1ms |

## Customization Guide

### 1. Database Configuration

Update the `DatabaseConfig` struct for your specific needs:

```rust
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub engine: DatabaseEngine,
    // Add your custom configuration fields
    pub pool_timeout: u64,
    pub ssl_mode: String,
    pub application_name: String,
}
```

### 2. Schema Customization

Replace the sample schema in `init_database_schema()`:

```rust
fn init_database_schema(&self) -> Result<()> {
    // Replace with your actual table schemas
    self.create_users_table()?;
    self.create_products_table()?;
    self.create_orders_table()?;
    // Add your custom tables
    Ok(())
}
```

### 3. Security Rules

Customize security validation in `validate_query_security()`:

```rust
fn validate_query_security(&self, query: &str) -> Result<(), String> {
    // Add your custom security patterns
    let dangerous_patterns = [
        "drop table",
        "delete from", 
        // Add patterns specific to your domain
        "your_sensitive_table",
        "admin_operations",
    ];
    
    // Add your custom validation logic
    if query.contains("sensitive_operation") && !self.is_admin_user() {
        return Err("Admin privileges required".to_string());
    }
    
    Ok(())
}
```

### 4. Custom MCP Tools

Add domain-specific tools using the `#[tool]` macro:

```rust
#[tool(description = "Get user profile information")]
async fn get_user_profile(
    &self,
    Parameters(args): Parameters<GetUserProfileArgs>,
) -> Result<CallToolResult, McpError> {
    // Your custom tool implementation
    self.update_stats("get_user_profile").await;
    
    // Execute your domain-specific query
    let query = "SELECT * FROM users WHERE id = $1";
    let result = self.execute_query_internal(query, &args.params).await?;
    
    Ok(CallToolResult::success(vec![Content::text(
        self.format_user_profile(&result)
    )]))
}
```

### 5. Connection Pool Setup

For production databases, implement actual connection pooling:

#### PostgreSQL with deadpool
```rust
use deadpool_postgres::{Config, Pool, Runtime};

impl DatabaseServer {
    fn init_postgres_pool(&self) -> Result<Pool> {
        let mut cfg = Config::new();
        cfg.host = Some("localhost".to_string());
        cfg.dbname = Some("mydb".to_string());
        cfg.user = Some("user".to_string());
        cfg.password = Some("pass".to_string());
        
        cfg.create_pool(Some(Runtime::Tokio1), NoTls)
    }
}
```

#### MySQL with mysql crate
```rust
use mysql::{Pool, OptsBuilder};

impl DatabaseServer {
    fn init_mysql_pool(&self) -> Result<Pool> {
        let opts = OptsBuilder::new()
            .ip_or_hostname(Some("localhost"))
            .db_name(Some("mydb"))
            .user(Some("user"))
            .pass(Some("pass"));
            
        Pool::new(opts)
    }
}
```

## Development Commands

The template includes a comprehensive `justfile` with development commands:

### Building and Running
```bash
just build                    # Build the template
just build-release            # Build for production
just run                      # Run with SQLite
just run-postgres             # Run with PostgreSQL
just run-mysql                # Run with MySQL
just develop                  # Run with debug logging
```

### Testing and Quality
```bash
just test                     # Run all tests
just test-security            # Run security tests
just test-integration         # Run integration tests
just clippy                   # Run linting
just format                   # Format code
just verify                   # Full CI pipeline
```

### Database Operations
```bash
just setup-sqlite             # Setup SQLite for testing
just setup-postgres           # Setup PostgreSQL database
just setup-mysql              # Setup MySQL database
just migrate-example          # Run example migrations
```

### Development Tools
```bash
just watch                    # Watch and run tests
just watch-run                # Watch and run server
just interactive              # Interactive development
just docs                     # Generate documentation
```

## Production Deployment

### 1. Environment Configuration

Set environment variables for production:

```bash
export DATABASE_URL="postgresql://user:pass@localhost/proddb"
export RUST_LOG="info"
export MAX_CONNECTIONS="50"
export CONNECTION_TIMEOUT="30"
```

### 2. Build for Production

```bash
# Optimize for production
just package

# The binary will be available at:
# target/release/database-server
```

### 3. Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/database-server /usr/local/bin/
EXPOSE 3000
CMD ["database-server"]
```

### 4. Health Monitoring

The server provides health endpoints:
- Use `get_server_status` tool for health checks
- Monitor `get_database_stats` for performance metrics
- Set up alerting based on connection pool usage

## Security Considerations

### Query Validation
- ✅ SQL injection protection with pattern detection
- ✅ Parameterized query enforcement
- ✅ Dangerous operation blocking (DROP, DELETE, etc.)
- ✅ Multiple statement prevention
- ✅ Comment stripping and validation

### Access Control
```rust
// Implement role-based access control
fn validate_user_permissions(&self, query: &str, user_role: &str) -> Result<(), String> {
    match user_role {
        "read_only" => {
            if !query.to_lowercase().trim().starts_with("select") {
                return Err("Read-only users can only execute SELECT queries".to_string());
            }
        }
        "admin" => {
            // Admins can execute any validated query
        }
        _ => {
            return Err("Unknown user role".to_string());
        }
    }
    Ok(())
}
```

### Data Protection
- Use environment variables for sensitive configuration
- Implement connection encryption (SSL/TLS)
- Add query logging and audit trails
- Sanitize error messages to prevent information leakage

## Performance Optimization

### Connection Pooling
```rust
// Configure pool size based on your load
DatabaseConfig {
    max_connections: 50,        // Adjust based on database limits
    connection_timeout: 30,     // Seconds
    pool_timeout: 10,           // Pool acquisition timeout
    // ...
}
```

### Query Optimization
- Use query execution plans (`get_query_plan` tool)
- Monitor slow queries with statistics
- Implement query result caching for read-heavy workloads
- Use prepared statements for repeated queries

### Monitoring
```rust
// Add custom metrics
self.query_stats.entry(query_type.to_string())
    .and_modify(|(count, total_time)| {
        *count += 1;
        *total_time += execution_time;
    })
    .or_insert((1, execution_time));
```

## Testing Strategy

### Unit Tests
```bash
# Test individual components
cargo test test_server_creation
cargo test test_execute_query_tool
cargo test test_security_validation
```

### Integration Tests
```bash
# Test database interactions
cargo test test_postgres_integration
cargo test test_mysql_integration
cargo test test_sqlite_integration
```

### Security Tests
```bash
# Test security protections
cargo test test_sql_injection_protection
cargo test test_dangerous_patterns
cargo test test_access_control
```

### Performance Tests
```bash
# Benchmark operations
cargo test --release bench_query_execution
cargo test --release bench_connection_pool
```

## Migration from Existing Systems

### From Raw SQL Applications
1. Replace direct database calls with MCP tool calls
2. Update query patterns to use the validation system
3. Migrate connection management to the template's pool
4. Add security validation for existing queries

### From ORM-based Applications
1. Extract SQL queries from ORM to direct SQL
2. Map ORM operations to MCP tools
3. Implement custom tools for complex ORM operations
4. Preserve transaction boundaries and relationships

## Contributing

1. Fork the template for your specific use case
2. Add your domain-specific MCP tools
3. Extend security validation for your requirements
4. Add integration tests for your database schema
5. Document your customizations

## Troubleshooting

### Common Issues

**Connection Refused**
```bash
# Check database is running
just db-info

# Verify connection string
echo $DATABASE_URL
```

**Permission Denied**
```bash
# Check database user permissions
# Verify SSL configuration for remote connections
```

**Query Validation Errors**
```bash
# Test query validation
just test-security

# Check security patterns in validate_query_security()
```

**Performance Issues**
```bash
# Monitor connection pool
just run -- --debug

# Check query execution times
# Review connection pool configuration
```

### Debug Mode
```bash
# Run with full debug logging
RUST_LOG=debug just develop

# Enable SQL query logging
RUST_LOG=sqlx=debug just run-postgres
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Support

- 📚 [MCP Protocol Documentation](https://modelcontextprotocol.io/)
- 🦀 [Rust Database Programming](https://docs.rs/)
- 🔧 [Template Issues](https://github.com/your-org/database-integration-template/issues)
- 💬 [Community Discussions](https://github.com/your-org/database-integration-template/discussions)

---

**Next Steps**: Customize the schema, add your domain-specific tools, and deploy to production! 🚀