# Task 11b: S3 Upload Proxy via V5

**Created:** 2026-03-04 19:59 HCMC
**Status:** COMPLETE
**Completed:** 2026-03-04 20:13 HCMC
**Phase:** E (P1 Features)

## Overview

DTV BE proxies file uploads to MCP V5 S3 upload endpoint. No AWS credentials in DTV BE.
Flow: `FE -> DTV BE (POST /upload) -> V5 (api_v5.ainext.vn/tools/s3_upload) -> AWS S3`

FE sends JSON with base64-encoded files (same pattern as admin-cms). DTV BE forwards to V5 with `X-API-Key` header. Returns uploaded file URLs.

## V5 S3 Upload API Contract

- **Endpoint:** `POST {V5_API_URL}/tools/s3_upload`
- **Auth:** `X-API-Key` header
- **Request body:**
  ```json
  {
    "action": "upload",
    "files": [{ "name": "photo.jpg", "content": "<base64>", "mimetype": "image/jpeg" }]
  }
  ```
- **Response:**
  ```json
  {
    "success": true,
    "message": "Files uploaded successfully",
    "data": {
      "uploadedFiles": [{ "mediaId": "...", "url": "...", "thumbUrl": "...", "type": "...", "format": "...", "size": 123 }],
      "totalFiles": 1,
      "successfulUploads": 1,
      "failedUploads": 0
    }
  }
  ```

## Checklist

### BE Implementation
- [x] 1. Create `src/upload/mod.rs` + `src/upload/routes.rs` -- Axum route `POST /upload`
- [x] 2. JWT auth via `x-access-token` header (reuse `AuthToken` extractor)
- [x] 3. Accept JSON body: `{ files: [{ name, content, mimetype }] }` (base64 content)
- [x] 4. Forward to V5 `/tools/s3_upload` with `X-API-Key` header, `action: "upload"`
- [x] 5. Return V5 response (URLs) back to FE
- [x] 6. Register route in `http_stream_server.rs`: `.route("/upload", post(upload_proxy_handler))`
- [x] 7. Register module in `main.rs`

### FE Integration
- [x] 8. Add `uploadFile()` + `uploadFiles()` + `uploadBase64()` methods to `mcpClient.ts`
- [x] 9. Helper: reads File to base64 via FileReader, sends JSON to `POST /upload`

### Build & Deploy
- [x] 10. `cargo build --release --target x86_64-unknown-linux-gnu` -- 119 tests, 0 failed
- [x] 11. Deploy binary to server, `pm2 restart`
- [x] 12. Smoke test: upload 1x1 PNG pixel, URL returned successfully

### Verification
- [x] 13. curl test: upload base64 image -> got S3 URL back (https://api.aiva.vn/uploads/images/...)
- [x] 14. Verify URL is accessible (curl returned HTTP 200, image/png, 70 bytes)
- [x] 15. Data URI prefix stripping works (data:image/png;base64,... -> raw base64)
- [x] 16. Auth guard: no token -> 401 "Thiếu token xác thực"
- [x] 17. Validation: empty files array -> 400 "Không có file nào để tải lên"

## Files to Create/Modify

| File | Action | Purpose |
|------|--------|---------|
| `src/upload/mod.rs` | Created | Module declaration |
| `src/upload/routes.rs` | Created (~400 lines) | `POST /upload` Axum route, V5 proxy, validation, tests |
| `src/main.rs` | Edited | Add `mod upload;` (gated behind http-stream) |
| `src/mcp/http_stream_server.rs` | Edited | Register `/upload` route + upload_proxy_handler |
| `dautruongvui-fe/src/lib/mcpClient.ts` | Edited | Add `uploadFile()`, `uploadFiles()`, `uploadBase64()` methods |

## Notes

- No multipart/form-data -- FE reads file to base64, sends as JSON (simpler, matches V5 API directly)
- Max file size: enforce 10MB base64 limit in route (V5 accepts up to ~50MB)
- 60s timeout for V5 upload call (shorter than textgen's 120s)
- No credit deduction for uploads (uploads are free, the tool using the image costs credits)

## Lessons Learned

1. **V5 requires MongoDB ObjectId for userId** when using API key auth. DTV uses PostgreSQL UUIDs, so we pass a fixed service ObjectId (`000000000000000000000001`) for V5's internal file organization.
2. **Cloudflare intercepts 502/504 responses** with its own error page ("error code: 502"). Return HTTP 200 with `success: false` for V5 upstream errors to ensure JSON reaches the FE.
3. **Data URI prefix stripping** is essential -- FE `FileReader.readAsDataURL()` produces `data:image/jpeg;base64,...` but V5 expects raw base64.

## Smoke Test Results

| Test | Result |
|------|--------|
| No auth token | 401 "Thiếu token xác thực" |
| Empty files array | 400 "Không có file nào để tải lên" |
| Raw base64 PNG upload | SUCCESS -- S3 URL returned |
| Data URI prefix PNG upload | SUCCESS -- prefix stripped, same S3 URL |
| Uploaded URL accessible | HTTP 200, image/png, 70 bytes |