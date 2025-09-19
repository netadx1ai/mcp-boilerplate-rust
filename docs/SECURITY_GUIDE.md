# MCP Server Security Guide

**Version**: 1.0  
**Last Updated**: January 18, 2025  
**SDK Version**: RMCP v0.6.3  
**Project**: mcp-boilerplate-rust

A comprehensive guide for implementing enterprise-grade security in MCP servers, covering authentication, authorization, input validation, data protection, and compliance requirements.

---

## Table of Contents

1. [Security Philosophy](#security-philosophy)
2. [Threat Model](#threat-model)
3. [Authentication](#authentication)
4. [Authorization](#authorization)
5. [Input Validation](#input-validation)
6. [Data Protection](#data-protection)
7. [Network Security](#network-security)
8. [Container Security](#container-security)
9. [Monitoring and Auditing](#monitoring-and-auditing)
10. [Compliance](#compliance)

---

## Security Philosophy

### Security-First Design

Every MCP server component must be designed with security as a primary concern:

- **Defense in Depth**: Multiple layers of security controls
- **Least Privilege**: Minimal access rights for users and services
- **Zero Trust**: Verify every request regardless of source
- **Fail Secure**: Systems fail to a secure state
- **Security by Default**: Secure configurations out of the box

### Security Requirements

Our security framework addresses these critical requirements:

- **Confidentiality**: Protect sensitive data from unauthorized access
- **Integrity**: Ensure data accuracy and prevent tampering
- **Availability**: Maintain service availability against attacks
- **Accountability**: Track and audit all security-relevant actions
- **Non-repudiation**: Prevent denial of actions taken

### Risk-Based Approach

Security controls are implemented based on risk assessment:

```rust
#[derive(Debug, Clone, Copy)]
pub enum SecurityLevel {
    Public,      // No authentication required
    Internal,    // Basic authentication required
    Sensitive,   // Strong authentication + authorization
    Classified,  // Multi-factor authentication + encryption
}

#[derive(Debug, Clone, Copy)]
pub enum DataClassification {
    Public,      // No protection required
    Internal,    // Standard protection
    Confidential,// Enhanced protection
    Restricted,  // Maximum protection
}

pub struct SecurityContext {
    pub level: SecurityLevel,
    pub data_classification: DataClassification,
    pub user_clearance: Option<String>,
    pub audit_required: bool,
}
```

---

## Threat Model

### Threat Landscape

MCP servers face various security threats:

#### 1. Input-Based Attacks
- **SQL Injection**: Malicious SQL code in user inputs
- **Command Injection**: OS command execution through inputs
- **Code Injection**: Script injection in templates or dynamic code
- **Path Traversal**: Unauthorized file system access
- **LDAP Injection**: Directory service manipulation

#### 2. Authentication Attacks
- **Brute Force**: Password guessing attacks
- **Credential Stuffing**: Using stolen credentials
- **Session Hijacking**: Stealing authentication tokens
- **Privilege Escalation**: Gaining unauthorized access levels

#### 3. Data Attacks
- **Data Exfiltration**: Unauthorized data extraction
- **Data Corruption**: Malicious data modification
- **Privacy Violations**: Accessing personal information
- **Intellectual Property Theft**: Stealing proprietary data

#### 4. Infrastructure Attacks
- **DDoS**: Overwhelming service with requests
- **Resource Exhaustion**: Consuming system resources
- **Container Escape**: Breaking out of container isolation
- **Supply Chain**: Compromising dependencies

### Threat Modeling Framework

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ThreatModel {
    pub assets: Vec<Asset>,
    pub threats: Vec<Threat>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub controls: Vec<SecurityControl>,
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub classification: DataClassification,
    pub value: AssetValue,
    pub location: AssetLocation,
}

#[derive(Debug, Clone)]
pub struct Threat {
    pub id: String,
    pub name: String,
    pub description: String,
    pub likelihood: RiskLevel,
    pub impact: RiskLevel,
    pub threat_actors: Vec<ThreatActor>,
}

#[derive(Debug, Clone)]
pub struct SecurityControl {
    pub id: String,
    pub name: String,
    pub control_type: ControlType,
    pub effectiveness: ControlEffectiveness,
    pub cost: ControlCost,
}

#[derive(Debug, Clone, Copy)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum ControlType {
    Preventive,
    Detective,
    Corrective,
    Deterrent,
}
```

---

## Authentication

### Multi-Factor Authentication

```rust
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: String,        // Subject (user ID)
    pub exp: usize,         // Expiration time
    pub iat: usize,         // Issued at
    pub iss: String,        // Issuer
    pub aud: String,        // Audience
    pub roles: Vec<String>, // User roles
    pub permissions: Vec<String>, // Specific permissions
    pub mfa_verified: bool, // Multi-factor authentication status
    pub session_id: String, // Session identifier
}

pub struct AuthenticationService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    token_expiry: Duration,
    mfa_provider: Box<dyn MfaProvider>,
    password_policy: PasswordPolicy,
}

#[async_trait]
pub trait MfaProvider {
    async fn generate_challenge(&self, user_id: &str) -> Result<MfaChallenge, AuthError>;
    async fn verify_response(&self, challenge: &MfaChallenge, response: &str) -> Result<bool, AuthError>;
}

impl AuthenticationService {
    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
        mfa_token: Option<String>,
    ) -> Result<AuthToken, AuthError> {
        // Step 1: Validate password
        let user = self.validate_password(username, password).await?;
        
        // Step 2: Check if MFA is required
        if user.mfa_enabled {
            match mfa_token {
                Some(token) => {
                    // Verify MFA token
                    let challenge = self.get_active_mfa_challenge(&user.id).await?;
                    if !self.mfa_provider.verify_response(&challenge, &token).await? {
                        return Err(AuthError::InvalidMfaToken);
                    }
                }
                None => {
                    // Generate MFA challenge
                    let challenge = self.mfa_provider.generate_challenge(&user.id).await?;
                    return Err(AuthError::MfaRequired { challenge });
                }
            }
        }
        
        // Step 3: Generate JWT token
        let session_id = Uuid::new_v4().to_string();
        let claims = AuthClaims {
            sub: user.id.clone(),
            exp: (chrono::Utc::now() + chrono::Duration::from_std(self.token_expiry).unwrap()).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            iss: "mcp-server".to_string(),
            aud: "mcp-clients".to_string(),
            roles: user.roles.clone(),
            permissions: user.get_permissions(),
            mfa_verified: user.mfa_enabled,
            session_id: session_id.clone(),
        };
        
        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        
        // Step 4: Create session
        self.create_session(&user.id, &session_id).await?;
        
        Ok(AuthToken {
            token,
            expires_at: claims.exp,
            user_id: user.id,
            roles: user.roles,
        })
    }
    
    pub async fn validate_token(&self, token: &str) -> Result<AuthClaims, AuthError> {
        // Decode and validate JWT
        let token_data = decode::<AuthClaims>(
            token,
            &self.decoding_key,
            &Validation::default(),
        )?;
        
        let claims = token_data.claims;
        
        // Verify session is still active
        if !self.is_session_active(&claims.session_id).await? {
            return Err(AuthError::SessionExpired);
        }
        
        // Check if user is still active
        if !self.is_user_active(&claims.sub).await? {
            return Err(AuthError::UserDisabled);
        }
        
        Ok(claims)
    }
    
    async fn validate_password(&self, username: &str, password: &str) -> Result<User, AuthError> {
        // Get user from database
        let user = self.get_user_by_username(username).await?
            .ok_or(AuthError::InvalidCredentials)?;
        
        // Check account status
        if !user.is_active {
            return Err(AuthError::AccountDisabled);
        }
        
        if user.is_locked() {
            return Err(AuthError::AccountLocked);
        }
        
        // Verify password
        if !self.verify_password(password, &user.password_hash)? {
            // Record failed attempt
            self.record_failed_login(&user.id).await?;
            return Err(AuthError::InvalidCredentials);
        }
        
        // Reset failed login attempts on successful authentication
        self.reset_failed_login_attempts(&user.id).await?;
        
        Ok(user)
    }
    
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AuthError> {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};
        
        let parsed_hash = PasswordHash::new(hash)?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Account is disabled")]
    AccountDisabled,
    #[error("Account is locked")]
    AccountLocked,
    #[error("MFA is required")]
    MfaRequired { challenge: MfaChallenge },
    #[error("Invalid MFA token")]
    InvalidMfaToken,
    #[error("Session has expired")]
    SessionExpired,
    #[error("User is disabled")]
    UserDisabled,
    #[error("Token validation error: {0}")]
    TokenError(#[from] jsonwebtoken::errors::Error),
}
```

### OAuth 2.0 Integration

```rust
use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenResponse, AuthUrl, TokenUrl,
};

pub struct OAuth2Service {
    client: oauth2::basic::BasicClient,
    redirect_url: RedirectUrl,
    scopes: Vec<Scope>,
}

impl OAuth2Service {
    pub fn new(
        client_id: ClientId,
        client_secret: Option<ClientSecret>,
        auth_url: AuthUrl,
        token_url: Option<TokenUrl>,
        redirect_url: RedirectUrl,
    ) -> Result<Self, OAuth2Error> {
        let client = oauth2::basic::BasicClient::new(
            client_id,
            client_secret,
            auth_url,
            token_url,
        ).set_redirect_uri(redirect_url.clone());
        
        Ok(Self {
            client,
            redirect_url,
            scopes: vec![
                Scope::new("openid".to_string()),
                Scope::new("profile".to_string()),
                Scope::new("email".to_string()),
            ],
        })
    }
    
    pub fn generate_auth_url(&self) -> (String, CsrfToken, PkceCodeChallenge) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        
        let (auth_url, csrf_token) = self.client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(self.scopes.clone())
            .set_pkce_challenge(pkce_challenge.clone())
            .url();
        
        (auth_url.to_string(), csrf_token, pkce_challenge)
    }
    
    pub async fn exchange_code(
        &self,
        authorization_code: AuthorizationCode,
        csrf_token: CsrfToken,
        pkce_verifier: oauth2::PkceCodeVerifier,
    ) -> Result<OAuth2Token, OAuth2Error> {
        let token_result = self.client
            .exchange_code(authorization_code)
            .set_pkce_verifier(pkce_verifier)
            .request_async(oauth2::reqwest::async_http_client)
            .await?;
        
        Ok(OAuth2Token {
            access_token: token_result.access_token().secret().clone(),
            expires_in: token_result.expires_in(),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            scope: token_result.scopes().map(|scopes| {
                scopes.iter().map(|s| s.to_string()).collect()
            }).unwrap_or_default(),
        })
    }
}
```

### Password Security

```rust
use argon2::{Argon2, PasswordHasher};
use rand::Rng;

pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digits: bool,
    pub require_special: bool,
    pub max_age_days: u32,
    pub history_count: usize,
    pub lockout_threshold: u32,
    pub lockout_duration: Duration,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_digits: true,
            require_special: true,
            max_age_days: 90,
            history_count: 12,
            lockout_threshold: 5,
            lockout_duration: Duration::from_secs(900), // 15 minutes
        }
    }
}

impl PasswordPolicy {
    pub fn validate_password(&self, password: &str) -> Result<(), PasswordError> {
        if password.len() < self.min_length {
            return Err(PasswordError::TooShort { min: self.min_length });
        }
        
        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(PasswordError::MissingUppercase);
        }
        
        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(PasswordError::MissingLowercase);
        }
        
        if self.require_digits && !password.chars().any(|c| c.is_numeric()) {
            return Err(PasswordError::MissingDigits);
        }
        
        if self.require_special && !password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
            return Err(PasswordError::MissingSpecialChars);
        }
        
        // Check against common passwords
        if self.is_common_password(password) {
            return Err(PasswordError::CommonPassword);
        }
        
        Ok(())
    }
    
    pub fn hash_password(&self, password: &str) -> Result<String, PasswordError> {
        let salt = self.generate_salt();
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| PasswordError::HashingFailed(e.to_string()))?;
        
        Ok(password_hash.to_string())
    }
    
    fn generate_salt(&self) -> argon2::password_hash::Salt {
        use argon2::password_hash::{rand_core::OsRng, SaltString};
        SaltString::generate(&mut OsRng).as_salt()
    }
    
    fn is_common_password(&self, password: &str) -> bool {
        // Check against a list of common passwords
        const COMMON_PASSWORDS: &[&str] = &[
            "password", "123456", "password123", "admin", "qwerty",
            "letmein", "welcome", "monkey", "dragon", "password1"
        ];
        
        COMMON_PASSWORDS.contains(&password.to_lowercase().as_str())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Password too short (minimum {min} characters)")]
    TooShort { min: usize },
    #[error("Password must contain uppercase letters")]
    MissingUppercase,
    #[error("Password must contain lowercase letters")]
    MissingLowercase,
    #[error("Password must contain digits")]
    MissingDigits,
    #[error("Password must contain special characters")]
    MissingSpecialChars,
    #[error("Password is too common")]
    CommonPassword,
    #[error("Password hashing failed: {0}")]
    HashingFailed(String),
}
```

---

## Authorization

### Role-Based Access Control (RBAC)

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: HashSet<Permission>,
    pub inherits_from: Vec<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Permission {
    pub resource: String,
    pub action: String,
    pub conditions: Vec<PermissionCondition>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PermissionCondition {
    pub attribute: String,
    pub operator: ConditionOperator,
    pub value: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    In,
    NotIn,
}

pub struct AuthorizationService {
    roles: HashMap<String, Role>,
    user_roles: HashMap<String, Vec<String>>,
    policy_engine: PolicyEngine,
}

impl AuthorizationService {
    pub async fn check_permission(
        &self,
        user_id: &str,
        resource: &str,
        action: &str,
        context: &AuthzContext,
    ) -> Result<bool, AuthzError> {
        // Get user roles
        let user_roles = self.user_roles.get(user_id)
            .ok_or(AuthzError::UserNotFound)?;
        
        // Collect all permissions for user (including inherited)
        let mut user_permissions = HashSet::new();
        for role_id in user_roles {
            if let Some(role) = self.roles.get(role_id) {
                self.collect_role_permissions(role, &mut user_permissions);
            }
        }
        
        // Check if any permission matches the request
        for permission in &user_permissions {
            if permission.resource == resource && permission.action == action {
                // Evaluate conditions
                if self.evaluate_conditions(&permission.conditions, context).await? {
                    return Ok(true);
                }
            }
        }
        
        // Check dynamic policies
        self.policy_engine.evaluate(user_id, resource, action, context).await
    }
    
    fn collect_role_permissions(&self, role: &Role, permissions: &mut HashSet<Permission>) {
        // Add direct permissions
        permissions.extend(role.permissions.clone());
        
        // Add inherited permissions
        for parent_role_id in &role.inherits_from {
            if let Some(parent_role) = self.roles.get(parent_role_id) {
                self.collect_role_permissions(parent_role, permissions);
            }
        }
    }
    
    async fn evaluate_conditions(
        &self,
        conditions: &[PermissionCondition],
        context: &AuthzContext,
    ) -> Result<bool, AuthzError> {
        for condition in conditions {
            if !self.evaluate_single_condition(condition, context).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    async fn evaluate_single_condition(
        &self,
        condition: &PermissionCondition,
        context: &AuthzContext,
    ) -> Result<bool, AuthzError> {
        let context_value = context.get_attribute(&condition.attribute)
            .ok_or_else(|| AuthzError::MissingAttribute(condition.attribute.clone()))?;
        
        match condition.operator {
            ConditionOperator::Equals => Ok(context_value == condition.value),
            ConditionOperator::NotEquals => Ok(context_value != condition.value),
            ConditionOperator::Contains => Ok(context_value.contains(&condition.value)),
            ConditionOperator::StartsWith => Ok(context_value.starts_with(&condition.value)),
            ConditionOperator::EndsWith => Ok(context_value.ends_with(&condition.value)),
            ConditionOperator::GreaterThan => {
                let context_num: f64 = context_value.parse()
                    .map_err(|_| AuthzError::InvalidAttributeType)?;
                let condition_num: f64 = condition.value.parse()
                    .map_err(|_| AuthzError::InvalidConditionValue)?;
                Ok(context_num > condition_num)
            }
            ConditionOperator::LessThan => {
                let context_num: f64 = context_value.parse()
                    .map_err(|_| AuthzError::InvalidAttributeType)?;
                let condition_num: f64 = condition.value.parse()
                    .map_err(|_| AuthzError::InvalidConditionValue)?;
                Ok(context_num < condition_num)
            }
            ConditionOperator::In => {
                let values: Vec<&str> = condition.value.split(',').collect();
                Ok(values.contains(&context_value.as_str()))
            }
            ConditionOperator::NotIn => {
                let values: Vec<&str> = condition.value.split(',').collect();
                Ok(!values.contains(&context_value.as_str()))
            }
        }
    }
}

#[derive(Debug)]
pub struct AuthzContext {
    attributes: HashMap<String, String>,
}

impl AuthzContext {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }
    
    pub fn add_attribute(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
    }
    
    pub fn get_attribute(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }
}

// Usage in MCP tools
#[tool]
async fn secure_tool(
    &self,
    auth_token: String,
    sensitive_operation: String,
) -> Result<String, ServerError> {
    // Authenticate user
    let claims = self.auth_service.validate_token(&auth_token).await
        .map_err(|e| ServerError::AuthenticationError { message: e.to_string() })?;
    
    // Build authorization context
    let mut context = AuthzContext::new();
    context.add_attribute("user_id".to_string(), claims.sub.clone());
    context.add_attribute("operation".to_string(), sensitive_operation.clone());
    context.add_attribute("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
    
    // Check authorization
    let authorized = self.authz_service
        .check_permission(&claims.sub, "sensitive_data", "read", &context)
        .await
        .map_err(|e| ServerError::AuthorizationError { message: e.to_string() })?;
    
    if !authorized {
        return Err(ServerError::PermissionDenied {
            action: "read sensitive data".to_string(),
        });
    }
    
    // Audit the access
    self.audit_service.log_access(AuditEvent {
        user_id: claims.sub,
        resource: "sensitive_data".to_string(),
        action: "read".to_string(),
        result: "success".to_string(),
        timestamp: chrono::Utc::now(),
        additional_data: Some(json!({
            "operation": sensitive_operation,
            "ip_address": context.get_attribute("ip_address")
        })),
    }).await;
    
    // Perform the operation
    self.perform_sensitive_operation(sensitive_operation).await
}
```

### Attribute-Based Access Control (ABAC)

```rust
pub struct PolicyEngine {
    policies: Vec<Policy>,
    attribute_provider: Box<dyn AttributeProvider>,
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rules: Vec<PolicyRule>,
    pub effect: PolicyEffect,
    pub priority: u32,
}

#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub condition: PolicyCondition,
}

#[derive(Debug, Clone)]
pub enum PolicyCondition {
    And(Vec<PolicyCondition>),
    Or(Vec<PolicyCondition>),
    Not(Box<PolicyCondition>),
    AttributeEquals { attribute: String, value: String },
    AttributeIn { attribute: String, values: Vec<String> },
    ResourceMatch { pattern: String },
    TimeWindow { start: String, end: String },
    Custom(String), // Custom condition expression
}

#[derive(Debug, Clone, Copy)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[async_trait]
pub trait AttributeProvider {
    async fn get_user_attributes(&self, user_id: &str) -> Result<Attributes, AttributeError>;
    async fn get_resource_attributes(&self, resource: &str) -> Result<Attributes, AttributeError>;
    async fn get_environment_attributes(&self) -> Result<Attributes, AttributeError>;
}

pub type Attributes = HashMap<String, AttributeValue>;

#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<AttributeValue>),
}

impl PolicyEngine {
    pub async fn evaluate(
        &self,
        user_id: &str,
        resource: &str,
        action: &str,
        context: &AuthzContext,
    ) -> Result<bool, AuthzError> {
        // Gather all attributes
        let user_attrs = self.attribute_provider.get_user_attributes(user_id).await?;
        let resource_attrs = self.attribute_provider.get_resource_attributes(resource).await?;
        let env_attrs = self.attribute_provider.get_environment_attributes().await?;
        
        // Combine all attributes
        let mut all_attributes = user_attrs;
        all_attributes.extend(resource_attrs);
        all_attributes.extend(env_attrs);
        
        // Add context attributes
        for (key, value) in &context.attributes {
            all_attributes.insert(key.clone(), AttributeValue::String(value.clone()));
        }
        
        // Evaluate policies in priority order
        let mut sorted_policies = self.policies.clone();
        sorted_policies.sort_by_key(|p| p.priority);
        
        for policy in sorted_policies {
            if self.evaluate_policy(&policy, &all_attributes).await? {
                return Ok(policy.effect == PolicyEffect::Allow);
            }
        }
        
        // Default deny
        Ok(false)
    }
    
    async fn evaluate_policy(
        &self,
        policy: &Policy,
        attributes: &Attributes,
    ) -> Result<bool, AuthzError> {
        for rule in &policy.rules {
            if !self.evaluate_condition(&rule.condition, attributes).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    async fn evaluate_condition(
        &self,
        condition: &PolicyCondition,
        attributes: &Attributes,
    ) -> Result<bool, AuthzError> {
        match condition {
            PolicyCondition::And(conditions) => {
                for cond in conditions {
                    if !self.evaluate_condition(cond, attributes).await? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            PolicyCondition::Or(conditions) => {
                for cond in conditions {
                    if self.evaluate_condition(cond, attributes).await? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            PolicyCondition::Not(condition) => {
                Ok(!self.evaluate_condition(condition, attributes).await?)
            }
            PolicyCondition::AttributeEquals { attribute, value } => {
                if let Some(attr_value) = attributes.get(attribute) {
                    match attr_value {
                        AttributeValue::String(s) => Ok(s == value),
                        AttributeValue::Number(n) => {
                            value.parse::<f64>().map(|v| *n == v).unwrap_or(false)
                        }
                        AttributeValue::Boolean(b) => {
                            value.parse::<bool>().map(|v| *b == v).unwrap_or(false)
                        }
                        _ => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            }
            PolicyCondition::AttributeIn { attribute, values } => {
                if let Some(attr_value) = attributes.get(attribute) {
                    match attr_value {
                        AttributeValue::String(s) => Ok(values.contains(s)),
                        AttributeValue::Array(arr) => {
                            for item in arr {
                                if let AttributeValue::String(s) = item {
                                    if values.contains(s) {
                                        return Ok(true);
                                    }
                                }
                            }
                            Ok(false)
                        }
                        _ => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            }
            PolicyCondition::ResourceMatch { pattern } => {
                if let Some(AttributeValue::String(resource)) = attributes.get("resource") {
                    Ok(self.match_pattern(pattern, resource))
                } else {
                    Ok(false)
                }
            }
            PolicyCondition::TimeWindow { start, end } => {
                let now = chrono::Utc::now().time();
                let start_time = chrono::NaiveTime::parse_from_str(start, "%H:%M:%S")
                    .map_err(|_| AuthzError::InvalidTimeFormat)?;
                let end_time = chrono::NaiveTime::parse_from_str(end, "%H:%M:%S")
                    .map_err(|_| AuthzError::InvalidTimeFormat)?;
                
                Ok(now >= start_time && now <= end_time)
            }
            PolicyCondition::Custom(expression) => {
                // Evaluate custom expression (simplified)
                self.evaluate_custom_expression(expression, attributes).await
            }
        }
    }
    
    fn match_pattern(&self, pattern: &str, value: &str) -> bool {
        // Simple glob pattern matching
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                value.starts_with(parts[0]) && value.ends_with(parts[1])
            } else {
                false
            }
        } else {
            pattern == value
        }
    }
    
    async fn evaluate_custom_expression(
        &self,
        _expression: &str,
        _attributes: &Attributes,
    ) -> Result<bool, AuthzError> {
        // Placeholder for custom expression evaluation
        // In practice, this could use a scripting engine like Lua or JavaScript
        Ok(false)
    }
}
```

---

## Input Validation

### Comprehensive Input Sanitization

```rust
use regex::Regex;
use once_cell::sync::Lazy;

// Pre-compiled security patterns
static SQL_INJECTION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(union|select|insert|update|delete|drop|create|alter|exec|script|declare|sp_|xp_)").unwrap()
});

static XSS_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)<script|javascript:|on\w+\s*=|data:text/html|vbscript:|expression\(").unwrap()
});

static COMMAND_INJECTION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[;&|`$(){}[\]\\]").unwrap()
});

static PATH_TRAVERSAL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\.\.(/|\\)|(/|\\)\.\.").unwrap()
});

pub struct InputValidator {
    max_string_length: usize,
    max_array_length: usize,
    allowed_characters: Regex,
    blocked_patterns: Vec<Regex>,
}

impl Default for InputValidator {
    fn default() -> Self {
        Self {
            max_string_length: 10000,
            max_array_length: 1000,
            allowed_characters: Regex::new(r"^[a-zA-Z0-9\s\-_.@!#$%^&*()+={}[\]:;\"'<>?,./\\]+$").unwrap(),
            blocked_patterns: vec![
                SQL_INJECTION_PATTERN.clone(),
                XSS_PATTERN.clone(),
                COMMAND_INJECTION_PATTERN.clone(),
                PATH_TRAVERSAL_PATTERN.clone(),
            ],
        }
    }
}

impl InputValidator {
    pub fn validate_string(&self, input: &str, field_name: &str) -> Result<String, ValidationError> {
        // Length validation
        if input.len() > self.max_string_length {
            return Err(ValidationError::TooLong {
                field: field_name.to_string(),
                max_length: self.max_string_length,
                actual_length: input.len(),
            });
        }
        
        // Character validation
        if !self.allowed_characters.is_match(input) {
            return Err(ValidationError::InvalidCharacters {
                field: field_name.to_string(),
            });
        }
        
        // Security pattern validation
        for pattern in &self.blocked_patterns {
            if pattern.is_match(input) {
                return Err(ValidationError::SecurityViolation {
                    field: field_name.to_string(),
                    reason: "Detected potentially malicious pattern".to_string(),
                });
            }
        }
        
        // Additional SQL injection checks
        if self.contains_sql_injection(input) {
            return Err(ValidationError::SqlInjection {
                field: field_name.to_string(),
            });
        }
        
        // HTML/Script injection checks
        if self.contains_xss(input) {
            return Err(ValidationError::XssAttempt {
                field: field_name.to_string(),
            });
        }
        
        Ok(self.sanitize_string(input))
    }
    
    pub fn validate_file_path(&self, path: &str) -> Result<String, ValidationError> {
        // Path traversal validation
        if PATH_TRAVERSAL_PATTERN.is_match(path) {
            return Err(ValidationError::PathTraversal);
        }
        
        // Null byte injection
        if path.contains('\0') {
            return Err(ValidationError::NullByteInjection);
        }
        
        // Validate path components
        let components: Vec<&str> = path.split('/').collect();
        for component in components {
            if component == ".." || component == "." {
                return Err(ValidationError::PathTraversal);
            }
        }
        
        // Length validation
        if path.len() > 4096 {
            return Err(ValidationError::PathTooLong);
        }
        
        Ok(path.to_string())
    }
    
    pub fn validate_email(&self, email: &str) -> Result<String, ValidationError> {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        
        if !email_regex.is_match(email) {
            return Err(ValidationError::InvalidEmail);
        }
        
        if email.len() > 254 { // RFC 5321 limit
            return Err(ValidationError::EmailTooLong);
        }
        
        Ok(email.to_lowercase())
    }
    
    pub fn validate_json(&self, json_str: &str) -> Result<serde_json::Value, ValidationError> {
        // Parse JSON
        let value: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| ValidationError::InvalidJson { error: e.to_string() })?;
        
        // Validate JSON structure
        self.validate_json_value(&value, 0)?;
        
        Ok(value)
    }
    
    fn validate_json_value(&self, value: &serde_json::Value, depth: usize) -> Result<(), ValidationError> {
        const MAX_DEPTH: usize = 32;
        
        if depth > MAX_DEPTH {
            return Err(ValidationError::JsonTooDeep);
        }
        
        match value {
            serde_json::Value::String(s) => {
                self.validate_string(s, "json_string")?;
            }
            serde_json::Value::Array(arr) => {
                if arr.len() > self.max_array_length {
                    return Err(ValidationError::ArrayTooLarge);
                }
                for item in arr {
                    self.validate_json_value(item, depth + 1)?;
                }
            }
            serde_json::Value::Object(obj) => {
                if obj.len() > 1000 { // Reasonable object size limit
                    return Err(ValidationError::ObjectTooLarge);
                }
                for (key, val) in obj {
                    self.validate_string(key, "json_key")?;
                    self.validate_json_value(val, depth + 1)?;
                }
            }
            _ => {} // Numbers, booleans, null are safe
        }
        
        Ok(())
    }
    
    fn contains_sql_injection(&self, input: &str) -> bool {
        // More specific SQL injection patterns
        let sql_keywords = [
            "union", "select", "insert", "update", "delete", "drop", "create", "alter",
            "exec", "execute", "sp_", "xp_", "declare", "cast", "convert", "char",
            "nchar", "varchar", "nvarchar", "sysobjects", "syscolumns", "information_schema"
        ];
        
        let input_lower = input.to_lowercase();
        
        // Check for SQL keywords followed by typical injection patterns
        for keyword in &sql_keywords {
            if input_lower.contains(keyword) {
                // Look for injection patterns after keywords
                if input_lower.contains(&format!("{} ", keyword)) ||
                   input_lower.contains(&format!("{}(", keyword)) ||
                   input_lower.contains(&format!("{}\t", keyword)) {
                    return true;
                }
            }
        }
        
        // Check for common injection techniques
        if input.contains("'") && (input.contains("or") || input.contains("and")) {
            return true;
        }
        
        false
    }
    
    fn contains_xss(&self, input: &str) -> bool {
        // HTML entities and encoded attacks
        let xss_patterns = [
            "&lt;script", "&amp;lt;script", "\\u003cscript", "\\x3cscript",
            "javascript:", "vbscript:", "data:text/html", "data:application/",
            "onload=", "onerror=", "onclick=", "onmouseover=", "onfocus=",
            "eval(", "expression(", "url(javascript:", "url(data:",
        ];
        
        let input_lower = input.to_lowercase();
        
        for pattern in &xss_patterns {
            if input_lower.contains(pattern) {
                return true;
            }
        }
        
        false
    }
    
    fn sanitize_string(&self, input: &str) -> String {
        // Remove null bytes
        let mut sanitized = input.replace('\0', "");
        
        // Normalize whitespace
        sanitized = sanitized.chars()
            .map(|c| if c.is_control() && c != '\n' && c != '\r' && c != '\t' { ' ' } else { c })
            .collect();
        
        // Trim excessive whitespace
        while sanitized.contains("  ") {
            sanitized = sanitized.replace("  ", " ");
        }
        
        sanitized.trim().to_string()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Field '{field}' is too long (max: {max_length}, actual: {actual_length})")]
    TooLong { field: String, max_length: usize, actual_length: usize },
    
    #[error("Field '{field}' contains invalid characters")]
    InvalidCharacters { field: String },
    
    #[error("Security violation in field '{field}': {reason}")]
    SecurityViolation { field: String, reason: String },
    
    #[error("SQL injection attempt detected in field '{field}'")]
    SqlInjection { field: String },
    
    #[error("XSS attempt detected in field '{field}'")]
    XssAttempt { field: String },
    
    #[error("Path traversal attempt detected")]
    PathTraversal,
    
    #[error("Null byte injection detected")]
    NullByteInjection,
    
    #[error("Path too long")]
    PathTooLong,
    
    #[error("Invalid email format")]
    InvalidEmail,
    
    #[error("Email too long")]
    EmailTooLong,
    
    #[error("Invalid JSON: {error}")]
    InvalidJson { error: String },
    
    #[error("JSON structure too deep")]
    JsonTooDeep,
    
    #[error("Array too large")]
    ArrayTooLarge,
    
    #[error("Object too large")]
    ObjectTooLarge,
}

// Usage in MCP tools
#[tool]
async fn secure_search(
    &self,
    auth_token: String,
    search_query: String,
    filters: Option<serde_json::Value>,
) -> Result<SearchResults, ServerError> {
    // Authenticate
    let claims = self.auth_service.validate_token(&auth_token).await?;
    
    // Validate and sanitize inputs
    let clean_query = self.input_validator
        .validate_string(&search_query, "search_query")
        .map_err(|e| ServerError::ValidationError {
            field: "search_query".to_string(),
            message: e.to_string(),
        })?;
    
    let clean_filters = if let Some(f) = filters {
        Some(self.input_validator
            .validate_json(&f.to_string())
            .map_err(|e| ServerError::ValidationError {
                field: "filters".to_string(),
                message: e.to_string(),
            })?)
    } else {
        None
    };
    
    // Check authorization
    let mut context = AuthzContext::new();
    context.add_attribute("user_id".to_string(), claims.sub.clone());
    context.add_attribute("operation".to_string(), "search".to_string());
    
    let authorized = self.authz_service
        .check_permission(&claims.sub, "search", "execute", &context)
        .await?;
    
    if !authorized {
        return Err(ServerError::PermissionDenied {
            action: "execute search".to_string(),
        });
    }
    
    // Perform secure search
    self.search_service.search(&clean_query, clean_filters.as_ref()).await
}
```

---

## Data Protection

### Encryption at Rest

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, NewAead}};
use argon2::Argon2;
use rand::{RngCore, thread_rng};

pub struct DataEncryption {
    cipher: Aes256Gcm,
    key_derivation: KeyDerivationService,
}

impl DataEncryption {
    pub fn new(master_key: &[u8]) -> Result<Self, EncryptionError> {
        if master_key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength);
        }
        
        let key = Key::from_slice(master_key);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self {
            cipher,
            key_derivation: KeyDerivationService::new(),
        })
    }
    
    pub fn encrypt_sensitive_data(&self, data: &[u8]) -> Result<EncryptedData, EncryptionError> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt data
        let ciphertext = self.cipher
            .encrypt(nonce, data)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        
        Ok(EncryptedData {
            nonce: nonce_bytes.to_vec(),
            ciphertext,
            algorithm: "AES256-GCM".to_string(),
            key_id: "master".to_string(),
        })
    }
    
    pub fn decrypt_sensitive_data(&self, encrypted: &EncryptedData) -> Result<Vec<u8>, EncryptionError> {
        if encrypted.algorithm != "AES256-GCM" {
            return Err(EncryptionError::UnsupportedAlgorithm(encrypted.algorithm.clone()));
        }
        
        let nonce = Nonce::from_slice(&encrypted.nonce);
        
        let plaintext = self.cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;
        
        Ok(plaintext)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub algorithm: String,
    pub key_id: String,
}

pub struct KeyDerivationService {
    argon2: Argon2<'static>,
}

impl KeyDerivationService {
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }
    
    pub fn derive_key(&self, password: &str, salt: &[u8]) -> Result<[u8; 32], EncryptionError> {
        use argon2::password_hash::{PasswordHasher, SaltString};
        
        let salt_string = SaltString::new(std::str::from_utf8(salt)
            .map_err(|_| EncryptionError::InvalidSalt)?)?;
        
        let hash = self.argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?;
        
        let hash_bytes = hash.hash.unwrap().as_bytes();
        if hash_bytes.len() < 32 {
            return Err(EncryptionError::InsufficientKeyMaterial);
        }
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash_bytes[..32]);
        Ok(key)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Invalid key length")]
    InvalidKeyLength,
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
    #[error("Invalid salt")]
    InvalidSalt,
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    #[error("Insufficient key material")]
    InsufficientKeyMaterial,
}
```

### Data Masking and Tokenization

```rust
use uuid::Uuid;
use std::collections::HashMap;

pub struct DataMaskingService {
    masking_rules: HashMap<String, MaskingRule>,
    tokenization_store: TokenizationStore,
}

#[derive(Debug, Clone)]
pub struct MaskingRule {
    pub field_pattern: String,
    pub masking_type: MaskingType,
    pub preserve_length: bool,
    pub preserve_format: bool,
}

#[derive(Debug, Clone)]
pub enum MaskingType {
    Full,           // Replace all characters with mask
    Partial { start: usize, end: usize }, // Mask middle, keep start/end
    FirstN(usize),  // Mask first N characters
    LastN(usize),   // Mask last N characters
    Email,          // Email-specific masking
    Phone,          // Phone number masking
    CreditCard,     // Credit card masking
    SSN,            // Social Security Number masking
}

impl DataMaskingService {
    pub fn new() -> Self {
        let mut masking_rules = HashMap::new();
        
        // Default masking rules
        masking_rules.insert("email".to_string(), MaskingRule {
            field_pattern: "email".to_string(),
            masking_type: MaskingType::Email,
            preserve_length: false,
            preserve_format: true,
        });
        
        masking_rules.insert("ssn".to_string(), MaskingRule {
            field_pattern: "ssn|social_security".to_string(),
            masking_type: MaskingType::SSN,
            preserve_length: true,
            preserve_format: true,
        });
        
        masking_rules.insert("credit_card".to_string(), MaskingRule {
            field_pattern: "credit_card|card_number|cc_num".to_string(),
            masking_type: MaskingType::CreditCard,
            preserve_length: true,
            preserve_format: true,
        });
        
        Self {
            masking_rules,
            tokenization_store: TokenizationStore::new(),
        }
    }
    
    pub fn mask_data(&self, data: &serde_json::Value) -> serde_json::Value {
        match data {
            serde_json::Value::Object(obj) => {
                let mut masked_obj = serde_json::Map::new();
                for (key, value) in obj {
                    let masked_value = if self.should_mask_field(key) {
                        self.mask_field_value(key, value)
                    } else {
                        self.mask_data(value) // Recursively process nested objects
                    };
                    masked_obj.insert(key.clone(), masked_value);
                }
                serde_json::Value::Object(masked_obj)
            }
            serde_json::Value::Array(arr) => {
                let masked_arr: Vec<_> = arr.iter().map(|v| self.mask_data(v)).collect();
                serde_json::Value::Array(masked_arr)
            }
            _ => data.clone(),
        }
    }
    
    fn should_mask_field(&self, field_name: &str) -> bool {
        let field_lower = field_name.to_lowercase();
        
        for rule in self.masking_rules.values() {
            let patterns: Vec<&str> = rule.field_pattern.split('|').collect();
            for pattern in patterns {
                if field_lower.contains(&pattern.to_lowercase()) {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn mask_field_value(&self, field_name: &str, value: &serde_json::Value) -> serde_json::Value {
        if let serde_json::Value::String(s) = value {
            let masked = self.mask_string(field_name, s);
            serde_json::Value::String(masked)
        } else {
            value.clone()
        }
    }
    
    fn mask_string(&self, field_name: &str, value: &str) -> String {
        let field_lower = field_name.to_lowercase();
        
        // Find applicable masking rule
        for rule in self.masking_rules.values() {
            let patterns: Vec<&str> = rule.field_pattern.split('|').collect();
            for pattern in patterns {
                if field_lower.contains(&pattern.to_lowercase()) {
                    return self.apply_masking_rule(value, &rule.masking_type);
                }
            }
        }
        
        // Default masking
        self.apply_masking_rule(value, &MaskingType::Partial { start: 2, end: 2 })
    }
    
    fn apply_masking_rule(&self, value: &str, masking_type: &MaskingType) -> String {
        match masking_type {
            MaskingType::Full => "*".repeat(value.len()),
            MaskingType::Partial { start, end } => {
                if value.len() <= start + end {
                    "*".repeat(value.len())
                } else {
                    let start_part = &value[..*start];
                    let end_part = &value[value.len() - end..];
                    let middle_len = value.len() - start - end;
                    format!("{}{}{}", start_part, "*".repeat(middle_len), end_part)
                }
            }
            MaskingType::FirstN(n) => {
                if value.len() <= *n {
                    "*".repeat(value.len())
                } else {
                    format!("{}{}", "*".repeat(*n), &value[*n..])
                }
            }
            MaskingType::LastN(n) => {
                if value.len() <= *n {
                    "*".repeat(value.len())
                } else {
                    format!("{}{}", &value[..value.len() - n], "*".repeat(*n))
                }
            }
            MaskingType::Email => {
                if let Some(at_pos) = value.find('@') {
                    let (local, domain) = value.split_at(at_pos);
                    if local.len() > 2 {
                        format!("{}****{}", &local[..1], domain)
                    } else {
                        format!("****{}", domain)
                    }
                } else {
                    "****@****.com".to_string()
                }
            }
            MaskingType::Phone => {
                let digits: String = value.chars().filter(|c| c.is_numeric()).collect();
                if digits.len() >= 10 {
                    format!("***-***-{}", &digits[digits.len()-4..])
                } else {
                    "***-***-****".to_string()
                }
            }
            MaskingType::CreditCard => {
                let digits: String = value.chars().filter(|c| c.is_numeric()).collect();
                if digits.len() >= 12 {
                    format!("****-****-****-{}", &digits[digits.len()-4..])
                } else {
                    "****-****-****-****".to_string()
                }
            }
            MaskingType::SSN => {
                let digits: String = value.chars().filter(|c| c.is_numeric()).collect();
                if digits.len() >= 9 {
                    format!("***-**-{}", &digits[digits.len()-4..])
                } else {
                    "***-**-****".to_string()
                }
            }
        }
    }
    
    pub fn tokenize_sensitive_data(&mut self, data: &str) -> Result<String, TokenizationError> {
        let token = format!("TOK_{}", Uuid::new_v4());
        self.tokenization_store.store_mapping(token.clone(), data.to_string())?;
        Ok(token)
    }
    
    pub fn detokenize_data(&self, token: &str) -> Result<String, TokenizationError> {
        self.tokenization_store.get_original_value(token)
    }
}

pub struct TokenizationStore {
    mappings: HashMap<String, String>,
    reverse_mappings: HashMap<String, String>,
}

impl TokenizationStore {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            reverse_mappings: HashMap::new(),
        }
    }
    
    pub fn store_mapping(&mut self, token: String, original: String) -> Result<(), TokenizationError> {
        // Check if original value already has a token
        if let Some(existing_token) = self.reverse_mappings.get(&original) {
            return Ok(());
        }
        
        self.mappings.insert(token.clone(), original.clone());
        self.reverse_mappings.insert(original, token);
        Ok(())
    }
    
    pub fn get_original_value(&self, token: &str) -> Result<String, TokenizationError> {
        self.mappings.get(token)
            .cloned()
            .ok_or(TokenizationError::TokenNotFound)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TokenizationError {
    #[error("Token not found")]
    TokenNotFound,
    #[error("Storage error: {0}")]
    StorageError(String),
}
```

---

## Network Security

### TLS Configuration

```rust
use rustls::{Certificate, PrivateKey, ServerConfig, ClientConfig};
use std::sync::Arc;

pub struct TlsConfigBuilder {
    min_protocol_version: Option<rustls::ProtocolVersion>,
    cipher_suites: Vec<rustls::SupportedCipherSuite>,
    cert_chain: Vec<Certificate>,
    private_key: Option<PrivateKey>,
}

impl TlsConfigBuilder {
    pub fn new() -> Self {
        Self {
            min_protocol_version: Some(rustls::ProtocolVersion::TLSv1_2),
            cipher_suites: vec![
                // Prefer AEAD cipher suites
                rustls::cipher_suite::TLS13_AES_256_GCM_SHA384,
                rustls::cipher_suite::TLS13_AES_128_GCM_SHA256,
                rustls::cipher_suite::TLS13_CHACHA20_POLY1305_SHA256,
                rustls::cipher_suite::TLS12_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
                rustls::cipher_suite::TLS12_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
            ],
            cert_chain: Vec::new(),
            private_key: None,
        }
    }
    
    pub fn with_min_protocol_version(mut self, version: rustls::ProtocolVersion) -> Self {
        self.min_protocol_version = Some(version);
        self
    }
    
    pub fn with_cert_chain(mut self, cert_chain: Vec<Certificate>) -> Self {
        self.cert_chain = cert_chain;
        self
    }
    
    pub fn with_private_key(mut self, private_key: PrivateKey) -> Self {
        self.private_key = Some(private_key);
        self
    }
    
    pub fn build_server_config(self) -> Result<Arc<ServerConfig>, TlsError> {
        let private_key = self.private_key.ok_or(TlsError::MissingPrivateKey)?;
        
        if self.cert_chain.is_empty() {
            return Err(Tl