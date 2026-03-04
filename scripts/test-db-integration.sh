#!/usr/bin/env bash
# PostgREST integration test for MCP db tool
# Requires: PostgREST running on localhost:3000 (docker-compose.postgrest.yml)
# Usage: ./scripts/test-db-integration.sh
set -euo pipefail

POSTGREST_URL="${POSTGREST_URL:-http://localhost:3000}"
PASS=0
FAIL=0
TOTAL=0

red()   { printf "\033[31m%s\033[0m" "$*"; }
green() { printf "\033[32m%s\033[0m" "$*"; }
bold()  { printf "\033[1m%s\033[0m" "$*"; }

assert_eq() {
  local label="$1" expected="$2" actual="$3"
  TOTAL=$((TOTAL + 1))
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
    echo "         actual: $haystack"
  fi
}

assert_not_empty() {
  local label="$1" value="$2"
  TOTAL=$((TOTAL + 1))
  if [ -n "$value" ] && [ "$value" != "null" ]; then
    PASS=$((PASS + 1))
    echo "  [PASS] $label"
  else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] $label (empty or null)"
  fi
}

assert_json_field() {
  local label="$1" json="$2" field="$3" expected="$4"
  local actual
  actual=$(echo "$json" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())$field)" 2>/dev/null || echo "__PARSE_ERROR__")
  assert_eq "$label" "$expected" "$actual"
}

# Convenience: POST/GET/PATCH/DELETE to PostgREST and return body
pgrest() {
  local method="$1" path="$2"
  shift 2
  curl -s -X "$method" "${POSTGREST_URL}${path}" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json" \
    "$@"
}

# ---------------------------------------------------------------------------
echo ""
bold "=== MCP DB Tool - PostgREST Integration Tests ==="
echo ""
echo "PostgREST URL: $POSTGREST_URL"
echo ""

# ---------------------------------------------------------------------------
# 0. Pre-flight: check PostgREST is reachable
# ---------------------------------------------------------------------------
bold "--- Pre-flight ---"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${POSTGREST_URL}/" 2>/dev/null || echo "000")
if [ "$HTTP_CODE" = "000" ]; then
  red "PostgREST not reachable at ${POSTGREST_URL}. Start it with:"
  echo ""
  echo "  docker compose -f docker-compose.postgrest.yml up -d"
  echo "  docker compose -f docker-compose.postgrest.yml exec -T postgres psql -U postgres -d myapp < scripts/postgrest-setup.sql"
  echo ""
  exit 1
fi
echo "  PostgREST is up (HTTP $HTTP_CODE)"
echo ""

# ---------------------------------------------------------------------------
# 1. Reset test data
# ---------------------------------------------------------------------------
bold "--- Reset test data ---"
# Delete all rows with id > 5 (cleanup from previous test runs)
pgrest DELETE "/test_mcp_db?id=gt.5" -H "Prefer: return=minimal" > /dev/null 2>&1 || true
echo "  Cleaned up leftover test rows"
echo ""

# ---------------------------------------------------------------------------
# 2. Query (SELECT)
# ---------------------------------------------------------------------------
bold "--- Test: Query (basic SELECT) ---"
RESULT=$(pgrest GET "/test_mcp_db?order=id.asc&limit=5")
COUNT=$(echo "$RESULT" | python3 -c "import sys,json; print(len(json.loads(sys.stdin.read())))" 2>/dev/null)
assert_eq "query returns 5 seed rows" "5" "$COUNT"

FIRST_NAME=$(echo "$RESULT" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())[0]['name'])" 2>/dev/null)
assert_eq "first row name is alpha" "alpha" "$FIRST_NAME"
echo ""

# ---------------------------------------------------------------------------
# 3. Query with filters
# ---------------------------------------------------------------------------
bold "--- Test: Query with filters ---"
RESULT=$(pgrest GET "/test_mcp_db?is_active=eq.true&order=id.asc")
COUNT=$(echo "$RESULT" | python3 -c "import sys,json; print(len(json.loads(sys.stdin.read())))" 2>/dev/null)
assert_eq "filter is_active=true returns 4 rows" "4" "$COUNT"

RESULT=$(pgrest GET "/test_mcp_db?value=gt.20&order=value.asc")
COUNT=$(echo "$RESULT" | python3 -c "import sys,json; print(len(json.loads(sys.stdin.read())))" 2>/dev/null)
assert_eq "filter value>20 returns 3 rows" "3" "$COUNT"

RESULT=$(pgrest GET "/test_mcp_db?name=like.a*")
COUNT=$(echo "$RESULT" | python3 -c "import sys,json; print(len(json.loads(sys.stdin.read())))" 2>/dev/null)
assert_eq "filter name like 'a%' returns 1 row" "1" "$COUNT"
echo ""

# ---------------------------------------------------------------------------
# 4. Query with select (column projection)
# ---------------------------------------------------------------------------
bold "--- Test: Query with column projection ---"
RESULT=$(pgrest GET "/test_mcp_db?select=id,name&limit=1")
KEYS=$(echo "$RESULT" | python3 -c "import sys,json; print(sorted(json.loads(sys.stdin.read())[0].keys()))" 2>/dev/null)
assert_eq "select id,name returns only those columns" "['id', 'name']" "$KEYS"
echo ""

# ---------------------------------------------------------------------------
# 5. Query with ordering
# ---------------------------------------------------------------------------
bold "--- Test: Query with ordering ---"
RESULT=$(pgrest GET "/test_mcp_db?order=value.desc&limit=1")
FIRST_VAL=$(echo "$RESULT" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())[0]['value'])" 2>/dev/null)
assert_eq "order by value desc, first is 50" "50" "$FIRST_VAL"
echo ""

# ---------------------------------------------------------------------------
# 6. Query with limit/offset
# ---------------------------------------------------------------------------
bold "--- Test: Query with limit/offset ---"
RESULT=$(pgrest GET "/test_mcp_db?order=id.asc&limit=2&offset=2")
COUNT=$(echo "$RESULT" | python3 -c "import sys,json; print(len(json.loads(sys.stdin.read())))" 2>/dev/null)
assert_eq "limit=2 offset=2 returns 2 rows" "2" "$COUNT"
FIRST_NAME=$(echo "$RESULT" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())[0]['name'])" 2>/dev/null)
assert_eq "offset=2 starts at gamma" "gamma" "$FIRST_NAME"
echo ""

# ---------------------------------------------------------------------------
# 7. Count via Prefer header
# ---------------------------------------------------------------------------
bold "--- Test: Count header ---"
HEADERS=$(curl -s -D - -o /dev/null "${POSTGREST_URL}/test_mcp_db" \
  -H "Prefer: count=exact" 2>/dev/null)
assert_contains "Content-Range header present" "Content-Range" "$HEADERS"
echo ""

# ---------------------------------------------------------------------------
# 8. Single row mode
# ---------------------------------------------------------------------------
bold "--- Test: Single row mode ---"
RESULT=$(curl -s "${POSTGREST_URL}/test_mcp_db?id=eq.1" \
  -H "Accept: application/vnd.pgrst.object+json" 2>/dev/null)
NAME=$(echo "$RESULT" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())['name'])" 2>/dev/null)
assert_eq "single row returns alpha" "alpha" "$NAME"
echo ""

# ---------------------------------------------------------------------------
# 9. Insert
# ---------------------------------------------------------------------------
bold "--- Test: Insert ---"
RESULT=$(pgrest POST "/test_mcp_db" \
  -H "Prefer: return=representation" \
  -d '{"name":"test_insert","value":999,"is_active":true}')
INS_ID=$(echo "$RESULT" | python3 -c "import sys,json; d=json.loads(sys.stdin.read()); print(d[0]['id'] if isinstance(d,list) else d['id'])" 2>/dev/null)
assert_not_empty "inserted row has id" "$INS_ID"

INS_NAME=$(echo "$RESULT" | python3 -c "import sys,json; d=json.loads(sys.stdin.read()); print(d[0]['name'] if isinstance(d,list) else d['name'])" 2>/dev/null)
assert_eq "inserted name is test_insert" "test_insert" "$INS_NAME"
echo ""

# ---------------------------------------------------------------------------
# 10. Batch Insert
# ---------------------------------------------------------------------------
bold "--- Test: Batch Insert ---"
RESULT=$(pgrest POST "/test_mcp_db" \
  -H "Prefer: return=representation" \
  -d '[{"name":"batch_a","value":100},{"name":"batch_b","value":200}]')
BATCH_COUNT=$(echo "$RESULT" | python3 -c "import sys,json; print(len(json.loads(sys.stdin.read())))" 2>/dev/null)
assert_eq "batch insert returns 2 rows" "2" "$BATCH_COUNT"
echo ""

# ---------------------------------------------------------------------------
# 11. Update
# ---------------------------------------------------------------------------
bold "--- Test: Update ---"
pgrest PATCH "/test_mcp_db?name=eq.test_insert" \
  -H "Prefer: return=representation" \
  -d '{"value":1000}' > /dev/null

RESULT=$(pgrest GET "/test_mcp_db?name=eq.test_insert")
UPD_VAL=$(echo "$RESULT" | python3 -c "import sys,json; print(json.loads(sys.stdin.read())[0]['value'])" 2>/dev/null)
assert_eq "updated value is 1000" "1000" "$UPD_VAL"
echo ""

# ---------------------------------------------------------------------------
# 12. Upsert
# ---------------------------------------------------------------------------
bold "--- Test: Upsert ---"
RESULT=$(curl -s -X POST "${POSTGREST_URL}/test_mcp_db" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -H "Prefer: return=representation,resolution=merge-duplicates" \
  -d "{\"id\":${INS_ID},\"name\":\"upserted\",\"value\":7777}")
UPS_NAME=$(echo "$RESULT" | python3 -c "import sys,json; d=json.loads(sys.stdin.read()); print(d[0]['name'] if isinstance(d,list) else d['name'])" 2>/dev/null)
assert_eq "upserted name is upserted" "upserted" "$UPS_NAME"
echo ""

# ---------------------------------------------------------------------------
# 13. Delete
# ---------------------------------------------------------------------------
bold "--- Test: Delete ---"
pgrest DELETE "/test_mcp_db?name=eq.upserted" -H "Prefer: return=minimal" > /dev/null
RESULT=$(pgrest GET "/test_mcp_db?name=eq.upserted")
DEL_COUNT=$(echo "$RESULT" | python3 -c "import sys,json; print(len(json.loads(sys.stdin.read())))" 2>/dev/null)
assert_eq "deleted row no longer exists" "0" "$DEL_COUNT"
echo ""

# ---------------------------------------------------------------------------
# 14. RPC (function call)
# ---------------------------------------------------------------------------
bold "--- Test: RPC (test_add function) ---"
RESULT=$(pgrest POST "/rpc/test_add" -d '{"a":17,"b":25}')
RPC_VAL=$(echo "$RESULT" | python3 -c "import sys,json; print(json.loads(sys.stdin.read()))" 2>/dev/null)
assert_eq "rpc test_add(17,25) = 42" "42" "$RPC_VAL"
echo ""

# ---------------------------------------------------------------------------
# 15. List tables (introspection via PostgREST root)
# ---------------------------------------------------------------------------
bold "--- Test: List tables (root endpoint) ---"
RESULT=$(curl -s "${POSTGREST_URL}/" 2>/dev/null)
assert_contains "root endpoint includes test_mcp_db" "test_mcp_db" "$RESULT"
echo ""

# ---------------------------------------------------------------------------
# 16. Describe table (via root OpenAPI spec)
# ---------------------------------------------------------------------------
bold "--- Test: Describe table (OpenAPI definitions) ---"
RESULT=$(curl -s "${POSTGREST_URL}/" 2>/dev/null | python3 -c "
import sys,json
d=json.loads(sys.stdin.read())
defn=d.get('definitions',{}).get('test_mcp_db',{})
print(json.dumps(defn))
" 2>/dev/null)
assert_contains "OpenAPI definition has id column" '"id"' "$RESULT"
assert_contains "OpenAPI definition has name column" '"name"' "$RESULT"
echo ""

# ---------------------------------------------------------------------------
# 17. Error cases
# ---------------------------------------------------------------------------
bold "--- Test: Error cases ---"
# Non-existent table
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${POSTGREST_URL}/nonexistent_table_xyz" 2>/dev/null)
assert_eq "non-existent table returns 404" "404" "$HTTP_CODE"

# Invalid filter
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${POSTGREST_URL}/test_mcp_db?id=badoperator.5" 2>/dev/null)
# PostgREST returns 400 for bad operators
TOTAL=$((TOTAL + 1))
if [ "$HTTP_CODE" = "400" ] || [ "$HTTP_CODE" = "404" ]; then
  PASS=$((PASS + 1))
  echo "  [PASS] bad filter returns error (HTTP $HTTP_CODE)"
else
  FAIL=$((FAIL + 1))
  echo "  [FAIL] bad filter should return 400/404, got $HTTP_CODE"
fi
echo ""

# ---------------------------------------------------------------------------
# 18. Cleanup
# ---------------------------------------------------------------------------
bold "--- Cleanup ---"
pgrest DELETE "/test_mcp_db?id=gt.5" -H "Prefer: return=minimal" > /dev/null 2>&1 || true
echo "  Removed test rows (id > 5)"
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