# Security Guidelines

**Last Updated:** 2026-01-08 (v0.3.1)

This document outlines security considerations and best practices for the MCP Boilerplate Rust project.

## Table of Contents

- [Security Features](#security-features)
- [Input Validation](#input-validation)
- [Environment Variables](#environment-variables)
- [Dependencies](#dependencies)
- [HTTP Mode Security](#http-mode-security)
- [Reporting Vulnerabilities](#reporting-vulnerabilities)

## Security Features

### Built-in Protections

1. **Input Validation**
   - Maximum message size: 10KB (10,240 bytes)
   - Empty message rejection
   - Type-safe parameter validation via serde

2. **Memory Safety**
   - Rust's ownership system prevents buffer overflows
   - No unsafe code in core implementation
   - Bounded allocations for all inputs

3. **Error Handling**
   - No sensitive data exposed in error messages
   - Generic error responses to prevent information leakage
   - Proper error propagation using Result types

## Input Validation

### Echo Tool Validation

The echo tool implements strict input validation:

```rust
// Maximum message size: 10KB
const MAX_MESSAGE_LENGTH: usize = 10 * 1024;

// Validation checks:
// 1. Message length <= 10KB
// 2. Message not empty
```

### Adding Validation to New Tools

When creating new tools, always validate inputs:

```rust
pub fn validate_input(data: &str) -> Result<(), McpError> {
    // Length check
    if data.len() > MAX_SIZE {
        return Err(McpError::InvalidParams(
            format!("Data too large: {} bytes", data.len())
        ));
    }
    
    // Empty check
    if data.is_empty() {
        return Err(McpError::InvalidParams(
            "Data cannot be empty".to_string()
        ));
    }
    
    Ok(())
}
```

## Environment Variables

### Secrets Management

**NEVER commit secrets to version control.**

1. Use `.env` for local development (already in .gitignore)
2. Use environment variables in production
3. Rotate secrets regularly

### Required Security Settings

Copy `.env.example` to `.env` and update:

```bash
# JWT Secret (if using 'auth' feature)
# Generate with: openssl rand -base64 32
JWT_SECRET=CHANGE_THIS_TO_STRONG_RANDOM_SECRET_MIN_32_CHARS

# CORS (HTTP mode)
# Never use '*' in production
CORS_ALLOWED_ORIGINS=https://yourdomain.com,https://app.yourdomain.com
```

### Generating Strong Secrets

```bash
# JWT Secret (minimum 32 characters)
openssl rand -base64 32

# Or using uuidgen
uuidgen
```

## Dependencies

### Security Auditing

Run security audits regularly:

```bash
# Install cargo-audit
cargo install cargo-audit

# Run audit
cargo audit

# Check advisories only
cargo audit --deny warnings
```

### Dependency Updates

```bash
# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update

# Rebuild with latest
cargo build --release
```

### Known Dependency Status

- **Core dependencies:** Actively maintained, no known vulnerabilities
- **rmcp v0.12:** Official MCP SDK, trusted source
- **axum v0.7:** Production-grade HTTP framework
- **tokio v1:** Industry-standard async runtime

### Duplicate Dependencies

Minor duplicates exist (thiserror v1 and v2) due to rmcp dependency. This is safe and expected.

## HTTP Mode Security

### When Using HTTP Feature

Enable security layers when running HTTP mode:

1. **Request Size Limits**
   ```rust
   // Add to HTTP server setup
   .layer(tower_http::limit::RequestBodyLimitLayer::new(
       1024 * 1024  // 1MB max
   ))
   ```

2. **Rate Limiting** (recommended for production)
   ```toml
   # Add to Cargo.toml
   tower-governor = "0.1"
   ```

3. **Timeouts**
   ```rust
   .layer(tower_http::timeout::TimeoutLayer::new(
       Duration::from_secs(30)
   ))
   ```

4. **CORS Configuration**
   - Development: `http://localhost:*`
   - Production: Specific domains only
   - Never use `*` in production

### HTTPS in Production

**Always use HTTPS in production.**

Options:
1. Reverse proxy (nginx/caddy) with TLS termination
2. Cloud load balancer with SSL certificate
3. Native TLS support (requires additional dependencies)

Example nginx configuration:

```nginx
server {
    listen 443 ssl http2;
    server_name api.yourdomain.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://127.0.0.1:8025;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## Authentication (Optional)

If using the `auth` feature:

### JWT Best Practices

1. **Strong Secrets**
   - Minimum 32 characters
   - Random generation
   - Rotate periodically

2. **Token Expiration**
   ```rust
   // Set reasonable expiration
   let exp = Utc::now() + Duration::hours(24);
   ```

3. **Secure Storage**
   - Store secrets in environment variables
   - Use secrets management service (AWS Secrets Manager, etc.)
   - Never log or expose tokens

4. **Validation**
   - Verify signature
   - Check expiration
   - Validate claims

## Deployment Security

### Production Checklist

- [ ] All secrets in environment variables (not code)
- [ ] HTTPS enabled (reverse proxy or native)
- [ ] CORS configured with specific origins
- [ ] Rate limiting enabled
- [ ] Request size limits configured
- [ ] Logging enabled (but no sensitive data logged)
- [ ] Dependencies audited (`cargo audit`)
- [ ] Error messages sanitized
- [ ] Firewall rules configured
- [ ] Regular security updates scheduled

### Stdio Mode

Stdio mode is inherently more secure:
- No network exposure
- Direct process communication
- Controlled by parent process

Use stdio mode when possible for maximum security.

## Logging Security

### Safe Logging Practices

```rust
// Good - no sensitive data
info!("Echo request received");

// Bad - exposes data
info!("Echo: password={}", password);  // NEVER DO THIS

// Good - sanitized
info!("Echo: {} bytes", message.len());
```

### Log Levels

- `error` - Critical issues only
- `warn` - Potential problems
- `info` - General operations (default)
- `debug` - Detailed debugging (development only)
- `trace` - Very verbose (never in production)

Production: `RUST_LOG=info`

## Common Vulnerabilities Mitigated

### What This Codebase Prevents

1. **Buffer Overflow** - Rust's memory safety
2. **SQL Injection** - No database queries (parameterized if added)
3. **XSS** - No HTML rendering
4. **CSRF** - Stateless API
5. **Memory Leaks** - Rust's ownership system
6. **Integer Overflow** - Bounded inputs

### What You Must Handle

1. **Rate Limiting** - Not implemented by default
2. **DDoS Protection** - Use reverse proxy or cloud provider
3. **API Key Management** - Implement if needed
4. **Audit Logging** - Add if compliance required

## Reporting Vulnerabilities

### Security Issues

If you discover a security vulnerability:

1. **DO NOT** open a public issue
2. Email: security@netadx.com (example)
3. Provide:
   - Description of vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if known)

### Response Timeline

- **24 hours:** Initial response
- **7 days:** Preliminary assessment
- **30 days:** Fix and disclosure

## Security Resources

### Tools

- `cargo audit` - Dependency vulnerability scanning
- `cargo deny` - License and security policy enforcement
- `cargo outdated` - Dependency updates
- `cargo clippy` - Linting and best practices

### References

- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Database](https://rustsec.org/)
- [MCP Specification](https://modelcontextprotocol.io/specification/2025-11-25)

## Updates

This security document is updated with each version release. Check the "Last Updated" timestamp at the top.

**Current Version:** v0.3.1  
**Security Status:** No known vulnerabilities

---

For questions or concerns, contact the maintainers or open a discussion on GitHub.