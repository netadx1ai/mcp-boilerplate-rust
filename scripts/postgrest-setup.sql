-- ============================================================================
-- PostgREST Setup for Đấu Trường Vui (DTV)
-- Run on the PostgreSQL server (52.221.127.152) as a superuser:
--   psql -h 52.221.127.152 -U postgres -d aivaweb < scripts/postgrest-setup.sql
--
-- PostgREST config (dtv.conf on BE server 163.44.193.56):
--   db-uri = "postgres://dtv_api:PASSWORD@52.221.127.152:5432/aivaweb"
--   db-schemas = "public"
--   db-anon-role = "dtv_api"
--   server-port = 3001
--   jwt-secret = "aivaAPI"
-- ============================================================================

BEGIN;

-- ============================================================================
-- 1. Create the dtv_api role (used by PostgREST)
-- ============================================================================

DO $$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'dtv_api') THEN
    CREATE ROLE dtv_api WITH LOGIN PASSWORD 'dtv_api_2026';
  END IF;
END
$$;

-- ============================================================================
-- 2. Grant schema access
-- ============================================================================

GRANT USAGE ON SCHEMA public TO dtv_api;

-- ============================================================================
-- 3. Grant CRUD on all existing dtv_ tables
-- ============================================================================

DO $$
DECLARE
  tbl TEXT;
BEGIN
  FOR tbl IN
    SELECT tablename FROM pg_tables
    WHERE schemaname = 'public' AND tablename LIKE 'dtv_%'
  LOOP
    EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON %I TO dtv_api;', tbl);
  END LOOP;
END
$$;

-- ============================================================================
-- 4. Grant sequence usage (for SERIAL/auto-increment columns)
-- ============================================================================

DO $$
DECLARE
  seq TEXT;
BEGIN
  FOR seq IN
    SELECT sequencename FROM pg_sequences
    WHERE schemaname = 'public' AND sequencename LIKE 'dtv_%'
  LOOP
    EXECUTE format('GRANT USAGE, SELECT ON SEQUENCE %I TO dtv_api;', seq);
  END LOOP;
END
$$;

-- ============================================================================
-- 5. Grant execute on all dtv_ functions
-- ============================================================================

DO $$
DECLARE
  func RECORD;
BEGIN
  FOR func IN
    SELECT p.proname, pg_get_function_identity_arguments(p.oid) AS args
    FROM pg_proc p
    JOIN pg_namespace n ON p.pronamespace = n.oid
    WHERE n.nspname = 'public' AND p.proname LIKE 'dtv_%'
  LOOP
    EXECUTE format('GRANT EXECUTE ON FUNCTION %I(%s) TO dtv_api;', func.proname, func.args);
  END LOOP;
END
$$;

-- ============================================================================
-- 6. Default privileges for future dtv_ objects
--    (applies to objects created by the current user)
-- ============================================================================

ALTER DEFAULT PRIVILEGES IN SCHEMA public
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO dtv_api;

ALTER DEFAULT PRIVILEGES IN SCHEMA public
  GRANT USAGE, SELECT ON SEQUENCES TO dtv_api;

ALTER DEFAULT PRIVILEGES IN SCHEMA public
  GRANT EXECUTE ON FUNCTIONS TO dtv_api;

-- ============================================================================
-- 7. Allow PostgREST authenticator to switch to dtv_api
--    (if using a separate authenticator role, grant it here)
-- ============================================================================

-- If PostgREST connects as dtv_api directly (db-anon-role = dtv_api),
-- no additional role switch is needed.
-- If using a separate authenticator role:
-- GRANT dtv_api TO authenticator;

-- ============================================================================
-- 8. Notify PostgREST to reload schema cache
-- ============================================================================

NOTIFY pgrst, 'reload schema';

COMMIT;