#!/usr/bin/env bash
# MCP DB Tool - Stdio Smoke Test
# Tests the actual MCP server binary with the db tool via JSON-RPC over stdio.
# Requires: PostgREST running on localhost:3000, binary built with --features postgres
# Usage: ./scripts/test-db-mcp-smoke.sh
set -euo pipefail

BINARY="${MCP_BINARY:-./target/debug/mcp-boilerplate-rust}"
POSTGREST_URL="${POSTGREST_URL:-http://localhost:3000}"
PASS=0
FAIL=0
TOTAL=0

red()   { printf "\033[31m%s\033[0m" "$*"; }
green() { printf "\033[32m%s\033[0m" "$*"; }
bold()  { printf "\033[1m%s\033[0m" "$*"; }

assert_json() {
  local label="$1" jq_filter="$2" expected="$3" json="$4"
  TOTAL=$((TOTAL + 1))
  local actual
  actual=$(echo "$json" | python3 -c "import sys,json; d=json.loads(sys.stdin.read()); print($jq_filter)" 2>/dev/null || echo "__PARSE_ERROR__")
  if [ "$expected" = "$actual" ]; then
    PASS=$((PASS + 1))
    echo "  [PASS] $label"
  else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] $label"
    echo "         expected: $expected"
    echo "         actual:   $actual"
  fi
}

assert_contains() {
  local label="$1" needle="$2" haystack="$3"
  TOTAL=$((TOTAL + 1))
  if echo "$haystack" | grep -q "$needle"; then
    PASS=$((PASS + 1))
    echo "  [PASS] $label"
  else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] $label"
    echo "         expected to contain: $needle"
    echo "         got: $(echo "$haystack" | head -c 200)"
  fi
}

# Send a single JSON-RPC request to the MCP server via stdio and capture the response.
# The server is started fresh for each call to avoid state issues.
# Uses sleep delays between messages so the async MCP server has time to process each one.
mcp_call() {
  local json_rpc="$1"
  local init_req='{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"smoke-test","version":"0.1.0"}}}'
  local init_notif='{"jsonrpc":"2.0","method":"notifications/initialized"}'

  # Send init, wait, then initialized notification, wait, then actual request, wait for response
  (
    printf '%s\n' "$init_req"
    sleep 0.3
    printf '%s\n' "$init_notif"
    sleep 0.3
    printf '%s\n' "$json_rpc"
    sleep 3
  ) | POSTGREST_URL="$POSTGREST_URL" RUST_LOG=off timeout 10 "$BINARY" --mode stdio 2>/dev/null \
    | tail -1
}

# Extract the inner db tool response text from the MCP tool call result.
# MCP wraps tool output in: {"result":{"content":[{"text":"..."}]}}
extract_db_response() {
  local mcp_response="$1"
  python3 -c "
import sys, json
d = json.loads(sys.argv[1])
# Navigate: result.content[0].text (which is itself JSON)
text = d.get('result',{}).get('content',[{}])[0].get('text','{}')
print(text)
" "$mcp_response" 2>/dev/null || echo "{}"
}

# ---------------------------------------------------------------------------
echo ""
bold "=== MCP DB Tool - Stdio Smoke Test ==="
echo ""

# Pre-flight checks
if [ ! -x "$BINARY" ]; then
  echo "Binary not found at $BINARY"
  echo "Build with: cargo build --features postgres"
  exit 1
fi

HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${POSTGREST_URL}/" 2>/dev/null || echo "000")
if [ "$HTTP_CODE" = "000" ]; then
  red "PostgREST not reachable at ${POSTGREST_URL}"
  echo ""
  exit 1
fi
echo "Binary:        $BINARY"
echo "PostgREST URL: $POSTGREST_URL (HTTP $HTTP_CODE)"
echo ""

# ---------------------------------------------------------------------------
# Clean up test data from previous runs
# ---------------------------------------------------------------------------
curl -s -X DELETE "${POSTGREST_URL}/test_mcp_db?name=like.smoke_*" \
  -H "Prefer: return=minimal" > /dev/null 2>&1 || true

# ---------------------------------------------------------------------------
# 1. list_tables
# ---------------------------------------------------------------------------
bold "--- 1. list_tables ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"db","arguments":{"action":"list_tables"}}}')
DB_RESP=$(extract_db_response "$RAW")
assert_json "success is true" "d.get('success',False)" "True" "$DB_RESP"
assert_contains "data includes test_mcp_db" "test_mcp_db" "$DB_RESP"
echo ""

# ---------------------------------------------------------------------------
# 2. describe
# ---------------------------------------------------------------------------
bold "--- 2. describe ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"db","arguments":{"action":"describe","table":"test_mcp_db"}}}')
DB_RESP=$(extract_db_response "$RAW")
assert_json "success is true" "d.get('success',False)" "True" "$DB_RESP"
assert_contains "has properties.id" '"id"' "$DB_RESP"
assert_contains "has properties.name" '"name"' "$DB_RESP"
echo ""

# ---------------------------------------------------------------------------
# 3. query (basic)
# ---------------------------------------------------------------------------
bold "--- 3. query (basic) ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"db","arguments":{"action":"query","table":"test_mcp_db","order":[{"column":"id","ascending":true}],"limit":5}}}')
DB_RESP=$(extract_db_response "$RAW")
assert_json "success is true" "d.get('success',False)" "True" "$DB_RESP"
assert_json "returns 5 rows" "len(d.get('data',[]))" "5" "$DB_RESP"
assert_json "first row is alpha" "d['data'][0]['name']" "alpha" "$DB_RESP"
assert_json "metadata.action is query" "d.get('metadata',{}).get('action','')" "query" "$DB_RESP"
echo ""

# ---------------------------------------------------------------------------
# 4. query with filters
# ---------------------------------------------------------------------------
bold "--- 4. query with filters ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"db","arguments":{"action":"query","table":"test_mcp_db","filters":{"is_active":{"eq":true}},"order":[{"column":"id","ascending":true}]}}}')
DB_RESP=$(extract_db_response "$RAW")
assert_json "success is true" "d.get('success',False)" "True" "$DB_RESP"
assert_json "returns 4 active rows" "len(d.get('data',[]))" "4" "$DB_RESP"
echo ""

# ---------------------------------------------------------------------------
# 5. query with select (column projection)
# ---------------------------------------------------------------------------
bold "--- 5. query with column projection ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"db","arguments":{"action":"query","table":"test_mcp_db","select":"id,name","limit":1}}}')
DB_RESP=$(extract_db_response "$RAW")
assert_json "success is true" "d.get('success',False)" "True" "$DB_RESP"
assert_json "only id and name columns" "sorted(d['data'][0].keys())" "['id', 'name']" "$DB_RESP"
echo ""

# ---------------------------------------------------------------------------
# 6. query with ordering (desc)
# ---------------------------------------------------------------------------
bold "--- 6. query with ordering ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"db","arguments":{"action":"query","table":"test_mcp_db","order":[{"column":"value","ascending":false}],"limit":1}}}')
DB_RESP=$(extract_db_response "$RAW")
assert_json "first row value is 50 (desc)" "d['data'][0]['value']" "50" "$DB_RESP"
echo ""

# ---------------------------------------------------------------------------
# 7. insert
# ---------------------------------------------------------------------------
bold "--- 7. insert ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"db","arguments":{"action":"insert","table":"test_mcp_db","data":{"name":"smoke_insert","value":777,"is_active":true}}}}')
DB_RESP=$(extract_db_response "$RAW")
assert_json "success is true" "d.get('success',False)" "True" "$DB_RESP"
assert_json "metadata.action is insert" "d.get('metadata',{}).get('action','')" "insert" "$DB_RESP"

# Verify the insert via direct PostgREST query
VERIFY=$(curl -s "${POSTGREST_URL}/test_mcp_db?name=eq.smoke_insert")
assert_contains "row exists in DB" "smoke_insert" "$VERIFY"
echo ""

# ---------------------------------------------------------------------------
# 8. batch insert
# ---------------------------------------------------------------------------
bold "--- 8. batch insert ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"db","arguments":{"action":"insert","table":"test_mcp_db","data":[{"name":"smoke_batch_a","value":100},{"name":"smoke_batch_b","value":200}]}}}')
DB_RESP=$(extract_db_response "$RAW")
assert_json "success is true" "d.get('success',False)" "True" "$DB_RESP"
echo ""

# ---------------------------------------------------------------------------
# 9. update
# ---------------------------------------------------------------------------
bold "--- 9. update ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"db","arguments":{"action":"update","table":"test_mcp_db","filters":{"name":{"eq":"smoke_insert"}},"data":{"value":888}}}}')
DB_RESP=$(extract_db_response "$RAW")
assert_json "success is true" "d.get('success',False)" "True" "$DB_RESP"

# Verify
VERIFY=$(curl -s "${POSTGREST_URL}/test_mcp_db?name=eq.smoke_insert" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())[0]['value'])" 2>/dev/null)
TOTAL=$((TOTAL + 1))
if [ "$VERIFY" = "888" ]; then
  PASS=$((PASS + 1))
  echo "  [PASS] value updated to 888"
else
  FAIL=$((FAIL + 1))
  echo "  [FAIL] expected value 888, got $VERIFY"
fi
echo ""

# ---------------------------------------------------------------------------
# 10. upsert
# ---------------------------------------------------------------------------
bold "--- 10. upsert ---"
# Get the id of smoke_insert for upsert
SMOKE_ID=$(curl -s "${POSTGREST_URL}/test_mcp_db?name=eq.smoke_insert&select=id" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())[0]['id'])" 2>/dev/null)
RAW=$(mcp_call "{\"jsonrpc\":\"2.0\",\"id\":10,\"method\":\"tools/call\",\"params\":{\"name\":\"db\",\"arguments\":{\"action\":\"upsert\",\"table\":\"test_mcp_db\",\"data\":{\"id\":${SMOKE_ID},\"name\":\"smoke_upserted\",\"value\":9999},\"conflict\":\"id\"}}}")
DB_RESP=$(extract_db_response "$RAW")
assert_json "success is true" "d.get('success',False)" "True" "$DB_RESP"

# Verify upsert
VERIFY=$(curl -s "${POSTGREST_URL}/test_mcp_db?id=eq.${SMOKE_ID}" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())[0]['name'])" 2>/dev/null)
TOTAL=$((TOTAL + 1))
if [ "$VERIFY" = "smoke_upserted" ]; then
  PASS=$((PASS + 1))
  echo "  [PASS] upserted name is smoke_upserted"
else
  FAIL=$((FAIL + 1))
  echo "  [FAIL] expected smoke_upserted, got $VERIFY"
fi
echo ""

# ---------------------------------------------------------------------------
# 11. delete
# ---------------------------------------------------------------------------
bold "--- 11. delete ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"db","arguments":{"action":"delete","table":"test_mcp_db","filters":{"name":{"eq":"smoke_upserted"}}}}}')
DB_RESP=$(extract_db_response "$RAW")
assert_json "success is true" "d.get('success',False)" "True" "$DB_RESP"

VERIFY=$(curl -s "${POSTGREST_URL}/test_mcp_db?name=eq.smoke_upserted" | python3 -c "import sys,json; print(len(json.loads(sys.stdin.read())))" 2>/dev/null)
TOTAL=$((TOTAL + 1))
if [ "$VERIFY" = "0" ]; then
  PASS=$((PASS + 1))
  echo "  [PASS] deleted row is gone"
else
  FAIL=$((FAIL + 1))
  echo "  [FAIL] expected 0 rows, got $VERIFY"
fi
echo ""

# ---------------------------------------------------------------------------
# 12. rpc
# ---------------------------------------------------------------------------
bold "--- 12. rpc (test_add) ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"db","arguments":{"action":"rpc","function_name":"test_add","params":{"a":17,"b":25}}}}')
DB_RESP=$(extract_db_response "$RAW")
assert_json "success is true" "d.get('success',False)" "True" "$DB_RESP"
assert_json "test_add(17,25) = 42" "d.get('data',0)" "42" "$DB_RESP"
echo ""

# ---------------------------------------------------------------------------
# 13. update without filters (should be rejected)
# ---------------------------------------------------------------------------
bold "--- 13. error: update without filters ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"db","arguments":{"action":"update","table":"test_mcp_db","data":{"value":0}}}}')
DB_RESP=$(extract_db_response "$RAW")
assert_json "success is false" "d.get('success',True)" "False" "$DB_RESP"
assert_contains "error mentions filters" "filter" "$DB_RESP"
echo ""

# ---------------------------------------------------------------------------
# 14. delete without filters (should be rejected)
# ---------------------------------------------------------------------------
bold "--- 14. error: delete without filters ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":14,"method":"tools/call","params":{"name":"db","arguments":{"action":"delete","table":"test_mcp_db"}}}')
DB_RESP=$(extract_db_response "$RAW")
assert_json "success is false" "d.get('success',True)" "False" "$DB_RESP"
assert_contains "error mentions filters" "filter" "$DB_RESP"
echo ""

# ---------------------------------------------------------------------------
# 15. raw_sql (should be rejected)
# ---------------------------------------------------------------------------
bold "--- 15. error: raw_sql rejected ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":15,"method":"tools/call","params":{"name":"db","arguments":{"action":"raw_sql","sql":"SELECT 1"}}}')
DB_RESP=$(extract_db_response "$RAW")
assert_json "success is false" "d.get('success',True)" "False" "$DB_RESP"
assert_contains "error mentions raw_sql" "raw_sql" "$DB_RESP"
echo ""

# ---------------------------------------------------------------------------
# 16. metadata has execution_time_ms
# ---------------------------------------------------------------------------
bold "--- 16. metadata fields ---"
RAW=$(mcp_call '{"jsonrpc":"2.0","id":16,"method":"tools/call","params":{"name":"db","arguments":{"action":"query","table":"test_mcp_db","limit":1}}}')
DB_RESP=$(extract_db_response "$RAW")
TOTAL=$((TOTAL + 1))
EXEC_TIME=$(echo "$DB_RESP" | python3 -c "import sys,json; d=json.loads(sys.stdin.read()); print(d.get('metadata',{}).get('execution_time_ms',-1))" 2>/dev/null)
if [ "$EXEC_TIME" != "-1" ] && [ "$EXEC_TIME" != "None" ] && [ "$EXEC_TIME" != "null" ]; then
  PASS=$((PASS + 1))
  echo "  [PASS] metadata.execution_time_ms present (${EXEC_TIME}ms)"
else
  FAIL=$((FAIL + 1))
  echo "  [FAIL] metadata.execution_time_ms missing"
fi

assert_json "metadata.timestamp present" "bool(d.get('metadata',{}).get('timestamp',''))" "True" "$DB_RESP"
echo ""

# ---------------------------------------------------------------------------
# Cleanup
# ---------------------------------------------------------------------------
bold "--- Cleanup ---"
curl -s -X DELETE "${POSTGREST_URL}/test_mcp_db?name=like.smoke_*" \
  -H "Prefer: return=minimal" > /dev/null 2>&1 || true
echo "  Removed smoke test rows"
echo ""

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo "============================================"
bold "Results: $PASS/$TOTAL passed"
if [ "$FAIL" -gt 0 ]; then
  red "  $FAIL FAILED"
  echo ""
  exit 1
else
  green "  All tests passed"
  echo ""
  exit 0
fi