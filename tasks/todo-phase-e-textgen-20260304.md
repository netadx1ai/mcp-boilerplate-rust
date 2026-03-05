# Phase E: TextGen V5 Proxy + S3 Upload

**Created:** 2026-03-04 19:08 HCMC
**Updated:** 2026-03-04 19:32 HCMC
**Status:** Task 11 COMPLETE, Task 11b Not Started

---

## Task 11: BE TextGen Tool (V5 Proxy) -- COMPLETE

### 11.1 Create `src/tools/textgen.rs` -- DONE
- [x] Define TextGenRequest struct (action, prompt, model_code, system_prompt, max_tokens, temperature, json_mode, response_format, toolId, token)
- [x] Implement V5 proxy: POST to `{V5_API_URL}/tools/text_generation` with `X-API-Key` header
- [x] Always send `bypassConsume: true` (DTV manages credits in PostgreSQL)
- [x] Pass `response_format` through to V5 as-is
- [x] If only `json_mode: true` without explicit `response_format`, default to `{ type: "json_object" }`
- [x] JWT token verification before calling V5
- [x] Credit check/deduct via PostgREST (dtv_tool_settings, dtv_credit_wallets, dtv_credit_usage)
- [x] Return V5 response data to caller

### 11.2 Register textgen in `src/tools/mod.rs` -- DONE
- [x] Add `pub mod textgen;` (gated behind `auth` feature)

### 11.3 Register textgen in `src/mcp/protocol_handler.rs` -- DONE
- [x] Add textgen to `handle_list_tools` (Tool definition with input_schema)
- [x] Add textgen to `handle_call_tool` match arm
- [x] Add `execute_textgen` method

### 11.4 Register textgen in `src/mcp/stdio_server.rs` -- DONE
- [x] Add `#[tool]` for textgen in McpServer impl
- [x] Update server info strings

### 11.5 Build & Test -- DONE
- [x] `cargo build --features full` -- zero errors (3 pre-existing warnings only)
- [x] `cargo test --features full` -- 107 passed, 0 failed, 1 ignored
- [x] Cross-compile for Linux -- 12MB binary
- [x] Deploy to 163.44.193.56 via rsync, PM2 restart with V5 env vars
- [x] Smoke tests passed (see below)

### Smoke Test Results (2026-03-04 19:30 HCMC)

| Test | Result |
|------|--------|
| Health check | OK (v0.1.0) |
| Tools list | 3 tools: db, auth, textgen |
| textgen simple (no toolId) | SUCCESS -- V5 returned quiz JSON, bypassed=true |
| textgen with toolId=arena_solo (cost=0) | SUCCESS -- creditsUsed=0, free tool |
| textgen with toolId=career_assessment (free_daily=1) 1st call | SUCCESS -- creditsUsed=0 (free daily) |
| textgen with toolId=career_assessment 2nd call | SUCCESS -- creditsUsed=10, remaining=0 (deducted from bonus) |
| textgen insufficient credits 3rd call | BLOCKED -- "Không đủ tín dụng. Cần 10, còn 0." |
| Usage tracking (dtv_credit_usage) | Row created with count incrementing |
| Transaction logging (dtv_credit_transactions) | Row created with negative amount |

### Bug Fix During Implementation
- **Issue:** Initial code had wrong column names for dtv_ tables
- **Fix:** Corrected to actual DTV schema: cost, free_daily_limit, is_active, user_id, date, count
- **Lesson:** Always check actual DB schema via PostgREST before coding

### Key Architecture Notes
- textgen.rs: 897 lines (including tests)
- V5 timeout: 120s (AI generation can be slow)
- Credit deduction order: bonus -> referral -> paid (same as /credits/deduct endpoint)
- PM2 must be started fresh (pm2 delete + pm2 start) when adding new env vars, not just restart

---

## Task 11b: BE S3 Upload Proxy (V5) -- NOT STARTED
- [ ] Add `POST /upload` Axum route in http_stream_server.rs
- [ ] Forward multipart data to `{V5_API_URL}/tools/s3_upload` with `X-API-Key`
- [ ] Return S3 URL from V5 response

---

## Key Reference

- V5 action: `generate_text`
- V5 URL: `http://api_v5.ainext.vn/tools/text_generation`
- V5 API Key: env `V5_API_KEY`, header `X-API-Key`
- Prompt format: flat `prompt` + `system_prompt` (not messages[] array)
- Default model: `gemini-2.5-pro`

