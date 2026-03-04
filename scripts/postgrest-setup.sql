-- PostgREST setup for MCP db tool development
-- Run after starting docker-compose.postgrest.yml:
--   docker compose -f docker-compose.postgrest.yml exec -T postgres \
--     psql -U postgres -d myapp < scripts/postgrest-setup.sql

-- Create the anonymous role used by PostgREST
DO $$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'web_anon') THEN
    CREATE ROLE web_anon NOLOGIN;
  END IF;
END
$$;

-- Grant usage on public schema
GRANT USAGE ON SCHEMA public TO web_anon;

-- Grant CRUD on all existing tables
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO web_anon;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO web_anon;

-- Auto-grant for future tables
ALTER DEFAULT PRIVILEGES IN SCHEMA public
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO web_anon;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
  GRANT USAGE, SELECT ON SEQUENCES TO web_anon;

-- Allow web_anon to execute functions
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO web_anon;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
  GRANT EXECUTE ON FUNCTIONS TO web_anon;

-- Let PostgREST switch to web_anon
GRANT web_anon TO postgres;

-- ==========================================================================
-- Test table for MCP db tool integration tests
-- ==========================================================================

CREATE TABLE IF NOT EXISTS test_mcp_db (
  id         SERIAL PRIMARY KEY,
  name       TEXT NOT NULL,
  value      INTEGER DEFAULT 0,
  metadata   JSONB DEFAULT '{}',
  is_active  BOOLEAN DEFAULT TRUE,
  tags       TEXT[] DEFAULT '{}',
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert sample rows (idempotent via ON CONFLICT)
INSERT INTO test_mcp_db (id, name, value, metadata, is_active, tags) VALUES
  (1, 'alpha', 10,  '{"color": "red"}',   TRUE,  '{tag1,tag2}'),
  (2, 'beta',  20,  '{"color": "blue"}',  TRUE,  '{tag2,tag3}'),
  (3, 'gamma', 30,  '{"color": "green"}', FALSE, '{tag1}'),
  (4, 'delta', 40,  '{"color": "red"}',   TRUE,  '{tag3}'),
  (5, 'epsilon', 50, '{"color": "blue"}', TRUE,  '{tag1,tag2,tag3}')
ON CONFLICT (id) DO UPDATE SET
  name      = EXCLUDED.name,
  value     = EXCLUDED.value,
  metadata  = EXCLUDED.metadata,
  is_active = EXCLUDED.is_active,
  tags      = EXCLUDED.tags;

-- Reset sequence to avoid conflicts on future inserts
SELECT setval('test_mcp_db_id_seq', (SELECT COALESCE(MAX(id), 0) FROM test_mcp_db));

-- ==========================================================================
-- Test RPC function
-- ==========================================================================

CREATE OR REPLACE FUNCTION test_add(a INTEGER, b INTEGER)
RETURNS INTEGER
LANGUAGE sql STABLE
AS $$
  SELECT a + b;
$$;

-- Notify PostgREST to reload schema cache
NOTIFY pgrst, 'reload schema';