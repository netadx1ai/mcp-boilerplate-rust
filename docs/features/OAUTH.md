# OAuth 2.1 Authorization (MCP Spec)

MCP-compliant OAuth 2.1 authorization for HTTP transport.

**Version:** 0.5.2  
**Last Updated:** 2026-01-09 HCMC  
**Spec:** MCP Authorization 2025-03-26

---

## Overview

Implements the MCP Authorization specification:
- OAuth 2.1 with PKCE (required for all clients)
- Authorization Server Metadata Discovery (RFC 8414)
- Dynamic Client Registration (RFC 7591)
- Token Introspection (RFC 7662)
- Token Revocation (RFC 7009)

---

## Quick Start

### Build

```bash
cargo build --release --features "http,auth"
```

### Run

```bash
OAUTH_ISSUER="http://localhost:8025" \
OAUTH_CLIENT_SECRET="your-secret" \
./target/release/mcp-boilerplate-rust --mode http
```

---

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/.well-known/oauth-authorization-server` | Server metadata |
| GET | `/oauth/authorize` | Authorization endpoint |
| POST | `/oauth/token` | Token endpoint |
| POST | `/oauth/register` | Dynamic client registration |
| POST | `/oauth/introspect` | Token introspection |
| POST | `/oauth/revoke` | Token revocation |

---

## Authorization Flows

### 1. Authorization Code + PKCE (User Auth)

For interactive applications where a user authorizes access.

```
Client                    MCP Server                  User
  |                           |                         |
  |-- GET /oauth/authorize -->|                         |
  |   (code_challenge, etc)   |                         |
  |                           |-- Login/Consent UI ---->|
  |                           |<-- User Approves -------|
  |<-- Redirect with code ----|                         |
  |                           |                         |
  |-- POST /oauth/token ----->|                         |
  |   (code + code_verifier)  |                         |
  |<-- Access Token ----------|                         |
```

**Step 1: Generate PKCE**

```javascript
// Generate code_verifier (43-128 chars)
const verifier = generateRandomString(64);

// Generate code_challenge = BASE64URL(SHA256(verifier))
const challenge = base64url(sha256(verifier));
```

**Step 2: Authorization Request**

```
GET /oauth/authorize
  ?response_type=code
  &client_id=mcp-public
  &redirect_uri=http://localhost:3000/callback
  &code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM
  &code_challenge_method=S256
  &scope=mcp:read%20mcp:write
  &state=abc123
```

**Step 3: Token Exchange**

```bash
curl -X POST http://localhost:8025/oauth/token \
  -d "grant_type=authorization_code" \
  -d "code=AUTH_CODE" \
  -d "redirect_uri=http://localhost:3000/callback" \
  -d "code_verifier=YOUR_VERIFIER" \
  -d "client_id=mcp-public"
```

**Response:**

```json
{
  "access_token": "a1b2c3d4...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "r1s2t3u4...",
  "scope": "mcp:read mcp:write"
}
```

### 2. Client Credentials (Server-to-Server)

For server-to-server communication without user involvement.

```bash
curl -X POST http://localhost:8025/oauth/token \
  -u "mcp-server:your-secret" \
  -d "grant_type=client_credentials" \
  -d "scope=mcp:read mcp:write"
```

Or with body params:

```bash
curl -X POST http://localhost:8025/oauth/token \
  -d "grant_type=client_credentials" \
  -d "client_id=mcp-server" \
  -d "client_secret=your-secret" \
  -d "scope=mcp:read"
```

**Response:**

```json
{
  "access_token": "a1b2c3d4...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "mcp:read mcp:write"
}
```

### 3. Refresh Token

```bash
curl -X POST http://localhost:8025/oauth/token \
  -d "grant_type=refresh_token" \
  -d "refresh_token=YOUR_REFRESH_TOKEN" \
  -d "client_id=mcp-public"
```

Note: OAuth 2.1 requires token rotation. Each refresh returns a new refresh token.

---

## Server Metadata Discovery

MCP clients should first discover server capabilities:

```bash
curl http://localhost:8025/.well-known/oauth-authorization-server
```

**Response:**

```json
{
  "issuer": "http://localhost:8025",
  "authorization_endpoint": "http://localhost:8025/authorize",
  "token_endpoint": "http://localhost:8025/token",
  "registration_endpoint": "http://localhost:8025/register",
  "introspection_endpoint": "http://localhost:8025/introspect",
  "revocation_endpoint": "http://localhost:8025/revoke",
  "response_types_supported": ["code"],
  "grant_types_supported": ["authorization_code", "client_credentials", "refresh_token"],
  "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post", "none"],
  "code_challenge_methods_supported": ["S256"],
  "scopes_supported": ["mcp:read", "mcp:write"]
}
```

---

## Dynamic Client Registration

Register a new client dynamically (RFC 7591):

```bash
curl -X POST http://localhost:8025/oauth/register \
  -H "Content-Type: application/json" \
  -d '{
    "client_name": "My MCP App",
    "redirect_uris": ["http://localhost:3000/callback"],
    "grant_types": ["authorization_code", "refresh_token"],
    "token_endpoint_auth_method": "none",
    "response_types": ["code"]
  }'
```

**Response:**

```json
{
  "client_id": "client_a1b2c3d4e5f6",
  "client_name": "My MCP App",
  "redirect_uris": ["http://localhost:3000/callback"],
  "grant_types": ["authorization_code", "refresh_token"],
  "token_endpoint_auth_method": "none",
  "response_types": ["code"]
}
```

For confidential clients:

```bash
curl -X POST http://localhost:8025/oauth/register \
  -H "Content-Type: application/json" \
  -d '{
    "client_name": "My Server App",
    "redirect_uris": [],
    "grant_types": ["client_credentials"],
    "token_endpoint_auth_method": "client_secret_basic"
  }'
```

---

## Using Access Tokens

Include the token in the Authorization header for all MCP requests:

```bash
curl http://localhost:8025/mcp \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

---

## Token Introspection

Check if a token is valid:

```bash
curl -X POST http://localhost:8025/oauth/introspect \
  -d "token=YOUR_ACCESS_TOKEN"
```

**Active Token Response:**

```json
{
  "active": true,
  "scope": "mcp:read mcp:write",
  "client_id": "mcp-public",
  "username": "mcp_user",
  "exp": 1768024810,
  "iat": 1767938410
}
```

**Inactive Token Response:**

```json
{
  "active": false
}
```

---

## Token Revocation

Revoke a token:

```bash
curl -X POST http://localhost:8025/oauth/revoke \
  -d "token=YOUR_ACCESS_TOKEN"
```

Always returns 200 OK (per spec).

---

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OAUTH_ISSUER` | No | `http://localhost:8025` | Base URL for OAuth endpoints |
| `OAUTH_CLIENT_SECRET` | No | `mcp_secret_2026` | Default client secret |
| `OAUTH_TOKEN_EXPIRY` | No | `3600` | Access token lifetime (seconds) |
| `OAUTH_REFRESH_TOKEN_EXPIRY` | No | `2592000` | Refresh token lifetime (30 days) |

---

## Default Clients

Two clients are pre-configured:

### Confidential Client (Server-to-Server)

```
client_id: mcp-server
client_secret: $OAUTH_CLIENT_SECRET
grant_types: client_credentials, refresh_token
scopes: mcp:read, mcp:write, mcp:admin
```

### Public Client (SPA/Mobile)

```
client_id: mcp-public
client_secret: (none)
grant_types: authorization_code, refresh_token
redirect_uris: http://localhost:3000/callback, http://127.0.0.1:3000/callback
scopes: mcp:read, mcp:write
```

---

## Scopes

| Scope | Description |
|-------|-------------|
| `mcp:read` | Read access to MCP resources |
| `mcp:write` | Write access to MCP resources |
| `mcp:admin` | Administrative access |

---

## Security Requirements (MCP Spec)

1. **PKCE Required** - All clients must use PKCE with S256
2. **Bearer Tokens** - Tokens must be in Authorization header only
3. **Token Rotation** - Refresh tokens are rotated on each use
4. **HTTPS Required** - Non-localhost redirect URIs must be HTTPS
5. **401 Unauthorized** - Returned when authorization required

---

## Error Responses

### OAuth Errors

```json
{
  "error": "invalid_grant",
  "error_description": "Authorization code expired"
}
```

| Error | Description |
|-------|-------------|
| `invalid_request` | Missing or invalid parameter |
| `invalid_client` | Unknown or invalid client |
| `invalid_grant` | Invalid code or refresh token |
| `unauthorized_client` | Client not allowed for grant type |
| `unsupported_grant_type` | Grant type not supported |
| `invalid_scope` | Requested scope not allowed |

### HTTP Status Codes

| Code | Usage |
|------|-------|
| 200 | Success |
| 302 | Authorization redirect |
| 400 | Invalid request |
| 401 | Unauthorized (token required/invalid) |
| 403 | Forbidden (insufficient scope) |

---

## Integration Example

### JavaScript (Browser)

```javascript
// 1. Generate PKCE
const verifier = crypto.randomUUID() + crypto.randomUUID();
const encoder = new TextEncoder();
const data = encoder.encode(verifier);
const hash = await crypto.subtle.digest('SHA-256', data);
const challenge = btoa(String.fromCharCode(...new Uint8Array(hash)))
  .replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');

// 2. Redirect to authorize
const authUrl = new URL('http://localhost:8025/oauth/authorize');
authUrl.searchParams.set('response_type', 'code');
authUrl.searchParams.set('client_id', 'mcp-public');
authUrl.searchParams.set('redirect_uri', 'http://localhost:3000/callback');
authUrl.searchParams.set('code_challenge', challenge);
authUrl.searchParams.set('code_challenge_method', 'S256');
authUrl.searchParams.set('scope', 'mcp:read mcp:write');
authUrl.searchParams.set('state', crypto.randomUUID());

// Store verifier for callback
sessionStorage.setItem('pkce_verifier', verifier);
window.location.href = authUrl.toString();

// 3. Handle callback
const params = new URLSearchParams(window.location.search);
const code = params.get('code');
const verifier = sessionStorage.getItem('pkce_verifier');

const response = await fetch('http://localhost:8025/oauth/token', {
  method: 'POST',
  headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
  body: new URLSearchParams({
    grant_type: 'authorization_code',
    code: code,
    redirect_uri: 'http://localhost:3000/callback',
    code_verifier: verifier,
    client_id: 'mcp-public'
  })
});

const tokens = await response.json();
```

### Python (Server)

```python
import requests

# Client credentials flow
response = requests.post(
    'http://localhost:8025/oauth/token',
    auth=('mcp-server', 'your-secret'),
    data={
        'grant_type': 'client_credentials',
        'scope': 'mcp:read mcp:write'
    }
)

tokens = response.json()
access_token = tokens['access_token']

# Use token
response = requests.post(
    'http://localhost:8025/rpc',
    headers={'Authorization': f'Bearer {access_token}'},
    json={'jsonrpc': '2.0', 'id': 1, 'method': 'tools/list', 'params': {}}
)
```

---

## vs Simple JWT Auth

| Feature | JWT Auth | OAuth 2.1 |
|---------|----------|-----------|
| Complexity | Simple | More complex |
| Standard | Custom | RFC compliant |
| Client types | Single | Public + Confidential |
| Dynamic registration | No | Yes |
| Token introspection | No | Yes |
| MCP compliant | No | Yes |
| Use case | Internal APIs | External/MCP clients |

Use **JWT Auth** for simple internal APIs.
Use **OAuth 2.1** for MCP-compliant external access.

---

## Testing

```bash
# Run OAuth tests
cargo test --features "http,auth" oauth

# Expected output
test middleware::oauth::tests::test_generate_token ... ok
test middleware::oauth::tests::test_generate_code ... ok
test middleware::oauth::tests::test_validate_scope ... ok
test middleware::oauth::tests::test_validate_redirect_uri ... ok
test middleware::oauth::tests::test_base64_decode ... ok
test middleware::oauth::tests::test_oauth_state ... ok
test middleware::oauth::tests::test_metadata ... ok
```

---

## Files

```
src/middleware/
├── mod.rs      # Module exports
├── auth.rs     # Simple JWT auth
└── oauth.rs    # OAuth 2.1 implementation (~1200 lines)
```
