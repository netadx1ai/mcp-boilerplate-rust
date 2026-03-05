# Lessons -- Phase E TextGen (2026-03-04)

## Lesson 1: Always verify own DB schema before coding

**What happened:** Wrote textgen.rs credit logic with wrong column names (credit_cost, free_per_day, is_enabled, user_session_id, usage_date, usage_count). PostgREST returned `column does not exist` errors at runtime.

**Root cause:** Assumed column names from another project instead of checking DTV's own migration SQL or querying PostgREST directly.

**Fix:** Corrected to actual DTV schema columns: cost, free_daily_limit, is_active, user_id, date, count.

**Rule:** Before writing any PostgREST query in Rust, always run:
```
curl -s http://localhost:3001/dtv_{table}?limit=1
```
to see actual column names. DTV owns its own `dtv_` schema -- never copy column names from other projects.

---

## Lesson 2: PM2 env vars require delete + start, not just restart

**What happened:** Added `V5_API_URL` and `V5_API_KEY` to ecosystem.config.cjs and ran `pm2 restart`. The process came up but env vars were not loaded -- textgen returned "V5_API_KEY not configured".

**Root cause:** `pm2 restart` does not reload env vars from ecosystem config. It only restarts the process with the same env it had before.

**Fix:** Must do `pm2 delete <name>` then `pm2 start ecosystem.config.cjs` to pick up new env vars.

**Rule:** When adding new env vars to PM2 ecosystem config:
```
pm2 delete <process_name>
pm2 start ecosystem.config.cjs
pm2 save
```

---

## Lesson 3: DTV is standalone -- no cross-project references in code or docs

**What happened:** Task tracker included a "DTV vs BDHN column mapping" table. This implies DTV depends on or relates to BDHN, which it does not.

**Root cause:** Used another project as initial reference during development, then left comparison artifacts in documentation.

**Fix:** Removed the mapping table. DTV schema is self-contained.

**Rule:** DTV is a standalone app with its own `dtv_` prefixed tables. Internal docs should only reference DTV's own schema. If another project was consulted during development, that's implementation history -- not something to document.