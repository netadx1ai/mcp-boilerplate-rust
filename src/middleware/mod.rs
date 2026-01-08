#[cfg(all(feature = "http", feature = "auth"))]
pub mod auth;

#[cfg(all(feature = "http", feature = "auth"))]
pub use auth::{auth_middleware, optional_auth_middleware, AuthMiddleware, Claims};