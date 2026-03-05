# Lessons: Phase E Task 11b -- S3 Upload Proxy

**Created:** 2026-03-04 20:14 HCMC

---

## 1. V5 requires MongoDB ObjectId for userId with API key auth

**Mistake:** Passed DTV PostgreSQL UUID as `userId` to V5 `/tools/s3_upload`. V5 tried to use it as MongoDB ObjectId and failed with "input must be a 24 character hex string".

**Root cause:** V5 uses MongoDB internally. When authenticating via `X-API-Key` (server-to-server), V5 can't extract userId from a JWT, so it requires `userId` explicitly -- and validates it as a MongoDB ObjectId (24-char hex).

**Fix:** Use a fixed service ObjectId (`000000000000000000000001`) for all DTV uploads. V5 only uses this for internal file organization in MongoDB, not for DTV access control.

**Rule:** When calling V5 APIs that expect MongoDB ObjectIds, never pass PostgreSQL UUIDs. Use a fixed service ID or convert appropriately.

---

## 2. Cloudflare intercepts 502/504 with its own error page

**Mistake:** Returned `StatusCode::BAD_GATEWAY` (502) from Rust when V5 returned an error. Cloudflare's proxy intercepted the 502 and replaced the JSON body with "error code: 502" plain text, making it impossible for FE to parse.

**Root cause:** Cloudflare Full SSL mode replaces certain error status codes (502, 504, etc.) with its own error pages.

**Fix:** Return HTTP 200 with `success: false` in JSON body for V5 upstream errors. Only use 4xx status codes for client-side validation errors (missing auth, bad input).

**Rule:** Behind Cloudflare, never return 502/503/504 from your API. Use 200 + `success: false` for upstream errors. Reserve 4xx for client errors.

---

## 3. Data URI prefix must be stripped before forwarding to V5

**Context:** FE `FileReader.readAsDataURL()` produces strings like `data:image/jpeg;base64,/9j/4AAQ...`. V5 expects raw base64 without the prefix.

**Fix:** `strip_data_uri_prefix()` function finds `;base64,` and returns everything after it. Handles both prefixed and raw base64 input.

**Rule:** Always strip data URI prefixes when proxying base64 content between services.

---

## 4. AuthToken is a tuple struct, not a named-field struct

**Mistake:** Tried to access `auth.user_id` on `AuthToken` which is defined as `pub struct AuthToken(pub Claims)`.

**Fix:** Destructure as `let AuthToken(claims) = auth;` then use `claims.sub` for user_id.

**Rule:** Check the actual struct definition before accessing fields. Tuple structs use `.0` or destructuring, not named field access.