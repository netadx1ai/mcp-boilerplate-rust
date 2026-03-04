# PostgreSQL Database Tool (via PostgREST)

Full documentation for the `db` MCP tool -- a PostgreSQL CRUD interface that wraps PostgREST HTTP API.

**Feature Flag:** `postgres`
**File:** `src/tools/db.rs` (~1700 lines including 56 unit tests)
**Last Updated:** 2026-03-04 HCMC

---

## Overview

The `db` tool translates MCP tool calls into PostgREST HTTP requests, providing AI agents with direct database access through a structured, validated interface.

```
AI Agent -> MCP Server (Rust) -> PostgREST (HTTP) -> PostgreSQL
```

No new dependencies are added -- the tool reuses the existing `reqwest` crate already in the project.

---

## Quick Start

### 1. Start PostgREST Stack

```bash
cd /Volumes/T7Shield/Work2026/mcp-boilerplate-rust

# Start PostgreSQL 16 + PostgREST v12.2.0
docker compose -f docker-compose.postgrest.yml up -d

# Seed database (creates web_anon role, test table, test function)
docker compose -f docker-compose.postgrest.yml exec -T postgres \
  psql -U postgres -d myapp < scripts/postgrest-setup.sql

# Verify
curl -s http://localhost:3000/test_mcp_db?limit=1
```

### 2. Build with Feature

```bash
cargo build --features postgres
```

### 3. Run

```bash
POSTGREST_URL=http://localhost:3000 cargo run --features postgres -- --mode stdio
```

### 4. Test

```bash
# Unit tests (56 db-specific)
cargo test --features postgres -- db::

# PostgREST integration tests (23 tests)
./scripts/test-db-integration.sh

# MCP stdio end-to-end tests (34 tests)
./scripts/test-db-mcp-smoke.sh
```

---

## Environment Variables

| Variable | Default | Required | Description |
|----------|---------|----------|-------------|
| `POSTGREST_URL` | `http://localhost:3000` | No | PostgREST base URL |
| `POSTGREST_ANON_KEY` | (none) | No | Bearer token for anonymous access |
| `POSTGREST_TIMEOUT` | `30` | No | Request timeout in seconds |
| `DB_ALLOWED_TABLES` | (none) | No | Comma-separated table whitelist (e.g. `users,orders,products`) |
| `DB_TABLE_PREFIX` | (none) | No | Only allow tables starting with this prefix (e.g. `app_`) |

If neither `DB_ALLOWED_TABLES` nor `DB_TABLE_PREFIX` is set, all tables are accessible.

When both are set, a table must match the whitelist AND start with the prefix.

---

## Actions

### query

Select rows from a table.

```json
{
  "action": "query",
  "table": "users",
  "select": "id,name,email",
  "filters": { "is_active": { "eq": true } },
  "order": [{ "column": "created_at", "ascending": false }],
  "limit": 10,
  "offset": 0,
  "options": { "count": "exact" }
}
```

Response:
```json
{
  "success": true,
  "data": [
    { "id": 1, "name": "Alice", "email": "alice@example.com" },
    { "id": 2, "name": "Bob", "email": "bob@example.com" }
  ],
  "count": 42,
  "metadata": {
    "execution_time_ms": 8,
    "timestamp": "2026-03-04T06:17:16Z",
    "action": "query",
    "table": "users",
    "affected_rows": 2
  }
}
```

Options:
- `select` -- column projection as string (`"id,name"`) or array (`["id","name"]`)
- `filters` -- see Filter Operators section below
- `order` -- array of `{ "column": "...", "ascending": true/false }` or `{ "column": "...", "direction": "asc/desc" }` or plain string `"column.asc"`
- `limit` -- max rows to return
- `offset` -- rows to skip
- `options.count` -- set to `"exact"` to get total count via Content-Range header
- `options.single` -- set to `true` to return a single object instead of an array

### insert

Insert one or more rows.

```json
{
  "action": "insert",
  "table": "users",
  "data": { "name": "Alice", "email": "alice@example.com" }
}
```

Batch insert:
```json
{
  "action": "insert",
  "table": "users",
  "data": [
    { "name": "Alice", "email": "alice@example.com" },
    { "name": "Bob", "email": "bob@example.com" }
  ]
}
```

Default return preference is `representation` (returns inserted rows). Set `options.return_pref` to `"minimal"` to skip returning data.

### update

Update rows matching filters. **Filters are required** to prevent mass updates.

```json
{
  "action": "update",
  "table": "users",
  "filters": { "id": { "eq": 42 } },
  "data": { "name": "Updated Name" }
}
```

### delete

Delete rows matching filters. **Filters are required** to prevent mass deletes.

```json
{
  "action": "delete",
  "table": "users",
  "filters": { "id": { "eq": 42 } }
}
```

### upsert

Insert or update on conflict. Specify the conflict column(s).

```json
{
  "action": "upsert",
  "table": "users",
  "data": { "id": 42, "name": "Bob", "email": "bob@example.com" },
  "conflict": "id"
}
```

Uses PostgREST `Prefer: resolution=merge-duplicates` header.

### rpc

Call a PostgreSQL function.

```json
{
  "action": "rpc",
  "function_name": "test_add",
  "params": { "a": 17, "b": 25 }
}
```

Response:
```json
{
  "success": true,
  "data": 42,
  "metadata": {
    "execution_time_ms": 3,
    "timestamp": "2026-03-04T06:17:16Z",
    "action": "rpc"
  }
}
```

### list_tables

List all accessible tables via the PostgREST root endpoint (OpenAPI spec).

```json
{
  "action": "list_tables"
}
```

### describe

Get the schema definition of a table. Extracts from the PostgREST OpenAPI spec `definitions` section.

```json
{
  "action": "describe",
  "table": "users"
}
```

Response data contains the JSON Schema for the table (properties, types, required fields, defaults).

### raw_sql (not supported)

Explicitly rejected. Returns a descriptive error suggesting to use `rpc` with PostgreSQL functions instead.

```json
{
  "action": "raw_sql",
  "sql": "SELECT 1"
}
```

Response:
```json
{
  "success": false,
  "error": "raw_sql is not supported in PostgREST mode. Use 'rpc' action with a database function instead."
}
```

---

## Filter Operators

14 Supabase-compatible operators:

| Operator | MCP Filter | PostgREST Query Param | Description |
|----------|-----------|----------------------|-------------|
| `eq` | `{ "name": { "eq": "alice" } }` | `name=eq.alice` | Equal |
| `neq` | `{ "status": { "neq": "deleted" } }` | `status=neq.deleted` | Not equal |
| `gt` | `{ "age": { "gt": 18 } }` | `age=gt.18` | Greater than |
| `gte` | `{ "age": { "gte": 18 } }` | `age=gte.18` | Greater than or equal |
| `lt` | `{ "price": { "lt": 100 } }` | `price=lt.100` | Less than |
| `lte` | `{ "price": { "lte": 100 } }` | `price=lte.100` | Less than or equal |
| `like` | `{ "name": { "like": "%test%" } }` | `name=like.*test*` | Pattern match (case-sensitive) |
| `ilike` | `{ "name": { "ilike": "%test%" } }` | `name=ilike.*test*` | Pattern match (case-insensitive) |
| `is` | `{ "deleted": { "is": null } }` | `deleted=is.null` | IS NULL / IS TRUE / IS FALSE |
| `in` | `{ "status": { "in": ["a","b"] } }` | `status=in.(a,b)` | In list |
| `not` | `{ "status": { "not": "deleted" } }` | `status=not.eq.deleted` | Negation |
| `contains` | `{ "tags": { "contains": ["a"] } }` | `tags=cs.{a}` | Array contains |
| `containedBy` | `{ "tags": { "containedBy": ["a","b"] } }` | `tags=cd.{a,b}` | Array contained by |
| `overlaps` | `{ "tags": { "overlaps": ["a"] } }` | `tags=ov.{a}` | Array overlaps |

### Simple Equality Shorthand

Non-object values are treated as equality filters:

```json
{ "filters": { "name": "alice" } }
```

is equivalent to:

```json
{ "filters": { "name": { "eq": "alice" } } }
```

### Null Shorthand

```json
{ "filters": { "deleted_at": null } }
```

is equivalent to:

```json
{ "filters": { "deleted_at": { "is": null } } }
```

---

## Response Format

All actions return a `DbResponse`:

```json
{
  "success": true,
  "data": null,
  "error": null,
  "count": null,
  "metadata": {
    "execution_time_ms": 12,
    "timestamp": "2026-03-04T06:17:16.304953+00:00",
    "action": "query",
    "table": "users",
    "affected_rows": 5
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `success` | boolean | Whether the operation succeeded |
| `data` | any / null | Result data (array, object, scalar, or null) |
| `error` | string / null | Error message if `success` is false |
| `count` | number / null | Total count (when `options.count = "exact"`) |
| `metadata.execution_time_ms` | number | Request duration in milliseconds |
| `metadata.timestamp` | string | ISO 8601 timestamp |
| `metadata.action` | string | The action that was executed |
| `metadata.table` | string / null | Target table name |
| `metadata.affected_rows` | number / null | Number of rows in the result |

---

## Security

### Table Name Validation

Table names are validated against `^[A-Za-z_][A-Za-z0-9_]*$`. Injection attempts like `../hack` or `users; DROP TABLE` are rejected.

### Table Access Control

Two mechanisms, applied before any request is sent to PostgREST:

1. **Whitelist** (`DB_ALLOWED_TABLES`): comma-separated list of allowed table names
2. **Prefix** (`DB_TABLE_PREFIX`): only tables starting with this prefix are allowed

When both are set, a table must satisfy both conditions.

### Mass Operation Prevention

`update` and `delete` actions require non-empty `filters`. Attempting to update or delete without filters returns:

```json
{
  "success": false,
  "error": "update requires at least one filter to prevent mass updates"
}
```

### Authentication

The `token` field in the request overrides the `POSTGREST_ANON_KEY` environment variable. This allows per-request JWT authentication for PostgREST Row Level Security (RLS).

```json
{
  "action": "query",
  "table": "users",
  "token": "eyJhbGciOiJIUzI1NiIs..."
}
```

---

## Docker Compose Reference

File: `docker-compose.postgrest.yml`

| Service | Image | Port | Credentials |
|---------|-------|------|-------------|
| postgres | postgres:16 | 5432 | user: `postgres`, pass: `postgres`, db: `myapp` |
| postgrest | postgrest/postgrest:v12.2.0 | 3000 | anon role: `web_anon`, JWT secret: `your-jwt-secret-at-least-32-chars-long!!` |

### Start

```bash
docker compose -f docker-compose.postgrest.yml up -d
docker compose -f docker-compose.postgrest.yml exec -T postgres \
  psql -U postgres -d myapp < scripts/postgrest-setup.sql
```

### Stop

```bash
docker compose -f docker-compose.postgrest.yml down      # keep data
docker compose -f docker-compose.postgrest.yml down -v    # remove data
```

### Schema Reload

After DDL changes (CREATE TABLE, ALTER TABLE, CREATE FUNCTION):

```bash
docker compose -f docker-compose.postgrest.yml exec -T postgres \
  psql -U postgres -d myapp -c "NOTIFY pgrst, 'reload schema';"
```

---

## Adding a New Table

```sql
-- 1. Create table
CREATE TABLE IF NOT EXISTS my_table (
  id         SERIAL PRIMARY KEY,
  name       TEXT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 2. Grants are automatic via ALTER DEFAULT PRIVILEGES in setup SQL.
--    If table was created before setup SQL ran:
GRANT SELECT, INSERT, UPDATE, DELETE ON my_table TO web_anon;
GRANT USAGE, SELECT ON my_table_id_seq TO web_anon;

-- 3. Reload PostgREST schema cache
NOTIFY pgrst, 'reload schema';
```

## Adding an RPC Function

```sql
CREATE OR REPLACE FUNCTION calculate_total(order_id INTEGER)
RETURNS NUMERIC
LANGUAGE sql STABLE
AS $$
  SELECT COALESCE(SUM(price * quantity), 0)
  FROM order_items
  WHERE order_items.order_id = calculate_total.order_id;
$$;

GRANT EXECUTE ON FUNCTION calculate_total TO web_anon;
NOTIFY pgrst, 'reload schema';
```

Call via: `{ "action": "rpc", "function_name": "calculate_total", "params": { "order_id": 123 } }`

---

## Claude Desktop Configuration

File: `examples/claude_desktop_config_db.json`

```json
{
  "mcpServers": {
    "mcp-boilerplate-rust-db": {
      "command": "/path/to/mcp-boilerplate-rust/target/release/mcp-boilerplate-rust",
      "args": ["--mode", "stdio"],
      "env": {
        "RUST_LOG": "off",
        "POSTGREST_URL": "http://localhost:3000",
        "POSTGREST_ANON_KEY": "",
        "POSTGREST_TIMEOUT": "30",
        "DB_ALLOWED_TABLES": "",
        "DB_TABLE_PREFIX": ""
      }
    }
  }
}
```

Config file location (macOS): `~/Library/Application Support/Claude/claude_desktop_config.json`

Binary MUST be built with `cargo build --release --features postgres`. Without the feature flag, calling the db tool returns a clear error.

---

## Implementation Details

### Architecture

Single file: `src/tools/db.rs`

```
DbRequest (JSON) -> build_request() -> PostgRestRequest -> HTTP -> normalize_response() -> DbResponse
                                                                           |
                                                              execute_db() post-processing
                                                              (describe: extract from OpenAPI)
```

Components:
- `PostgRestConfig` -- reads env vars, provides `is_table_allowed()` and `validate_table_name()`
- `DbRequest` / `DbResponse` -- serde types with `schemars::JsonSchema` for MCP schema generation
- `translate_filters()` -- converts `{ "col": { "op": val } }` to PostgREST query params
- `translate_order()` -- converts order array to PostgREST `order` param
- `translate_select()` -- converts select string/array to PostgREST `select` param
- `build_request()` -- dispatches by action, creates method/path/headers/body
- `normalize_response()` -- HTTP response to DbResponse with Content-Range parsing
- `execute_db()` -- orchestrator with post-processing for `describe` action

### Lazy Globals

```rust
static DB_CLIENT: OnceLock<Client> = OnceLock::new();
static DB_CONFIG: OnceLock<PostgRestConfig> = OnceLock::new();
```

Initialized on first call via `get_client()` / `get_config()`. The HTTP client timeout comes from `POSTGREST_TIMEOUT`.

### Feature Gate Pattern

The `#[tool_router]` rmcp macro does not support `#[cfg]` on individual tool methods. The `db` tool method is always compiled, but the method body is feature-gated:

```rust
#[tool(description = "...")]
async fn db(&self, Parameters(req): Parameters<Value>) -> Result<String, McpError> {
    #[cfg(feature = "postgres")]
    { /* actual implementation */ }

    #[cfg(not(feature = "postgres"))]
    { Err(McpError::invalid_params("Feature not enabled. Rebuild with: cargo build --features postgres", None)) }
}
```

### PostgREST v12 Describe Workaround

PostgREST v12 returns an empty body for OPTIONS requests on table endpoints. The `describe` action instead:
1. GETs the root `/` endpoint (returns the OpenAPI spec)
2. Post-processes in `execute_db()` to extract `definitions.{table_name}`

---

## Testing

### Unit Tests (56 tests in `src/tools/db.rs`)

```bash
cargo test --features postgres -- db::
```

Covers: config defaults, table allowed logic, table name validation, 14 filter operators, filter edge cases (null shorthand, simple equality, legacy format), order translation, select translation, all 8 action builders, error cases (missing table, missing data, missing function, table not allowed, invalid name, raw_sql rejection, unknown action), request serialization, response serialization, Content-Range parsing.

### PostgREST Integration Tests (23 tests)

```bash
./scripts/test-db-integration.sh
```

Requires running PostgREST. Tests: basic query, filters (eq/gt/like), column projection, ordering, limit/offset, count header, single row mode, insert, batch insert, update, upsert, delete, rpc, list_tables, describe (OpenAPI), error cases (404, bad filter).

### MCP Stdio Smoke Tests (34 tests)

```bash
./scripts/test-db-mcp-smoke.sh
```

Requires running PostgREST and built binary with `--features postgres`. Tests the full MCP JSON-RPC flow via stdio: list_tables, describe, query (basic/filters/select/order), insert, batch insert, update, upsert, delete, rpc, error cases (update/delete without filters, raw_sql), metadata fields (execution_time_ms, timestamp).

---

## Troubleshooting

### "Feature not enabled"

Binary was built without `--features postgres`. Rebuild:
```bash
cargo build --release --features postgres
```

### "PostgREST connection failed"

PostgREST is not running or `POSTGREST_URL` is wrong:
```bash
curl -s http://localhost:3000/
docker compose -f docker-compose.postgrest.yml up -d
```

### "PostgREST request timed out"

Increase timeout:
```bash
export POSTGREST_TIMEOUT=60
```

### "Table 'x' is not in the allowed tables list"

Table is blocked by `DB_ALLOWED_TABLES` or `DB_TABLE_PREFIX`. Check env vars or add the table to the whitelist.

### "update requires at least one filter"

Filters are mandatory for update and delete to prevent mass operations. Add a filter:
```json
{ "action": "update", "table": "users", "filters": { "id": { "eq": 42 } }, "data": { "name": "New" } }
```

### PostgREST returns 404 for a table

Table doesn't exist or `web_anon` has no access:
```bash
# Check table exists
docker compose -f docker-compose.postgrest.yml exec -T postgres \
  psql -U postgres -d myapp -c "\dt"

# Re-run setup SQL (grants access to all current and future tables)
docker compose -f docker-compose.postgrest.yml exec -T postgres \
  psql -U postgres -d myapp < scripts/postgrest-setup.sql
```

### PostgREST doesn't see new tables/functions

Reload schema cache:
```bash
docker compose -f docker-compose.postgrest.yml exec -T postgres \
  psql -U postgres -d myapp -c "NOTIFY pgrst, 'reload schema';"
```

---

## Production Considerations

- Change PostgreSQL password from `postgres` to a strong secret
- Change PostgREST JWT secret from the dev default
- Remove external port bindings (5432, 3000) -- expose only via internal network
- Set `PGRST_OPENAPI_MODE: disabled` to hide schema from the root endpoint
- Use PostgreSQL Row Level Security (RLS) for per-user data isolation
- Set `DB_ALLOWED_TABLES` to an explicit whitelist
- Set `POSTGREST_TIMEOUT` to 5-10s (prevent slow queries from blocking)
- Put PostgREST behind a reverse proxy with rate limiting
- Monitor `pg_stat_activity` for connection leaks