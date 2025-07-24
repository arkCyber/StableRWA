// =====================================================================================
// File: core-security/src/jwt.rs
// Description: JWT token management and validation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{SecurityError, UserClaims};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::info;
use uuid::Uuid;

/// JWT token manager
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    algorithm: Algorithm,
    access_token_expiry: Duration,
    refresh_token_expiry: Duration,
    issuer: String,
    audience: String,
}

impl JwtManager {
    /// Create new JWT manager with secret key
    pub fn new(
        secret: &str,
        access_token_hours: i64,
        refresh_token_days: i64,
        issuer: String,
        audience: String,
    ) -> Result<Self, SecurityError> {
        if secret.len() < 32 {
            return Err(SecurityError::ValidationError(
                "JWT secret must be at least 32 characters long".to_string(),
            ));
        }

        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());

        Ok(Self {
            encoding_key,
            decoding_key,
            algorithm: Algorithm::HS256,
            access_token_expiry: Duration::hours(access_token_hours),
            refresh_token_expiry: Duration::days(refresh_token_days),
            issuer,
            audience,
        })
    }

    /// Generate access token
    pub fn generate_access_token(&self, user_claims: &UserClaims) -> Result<String, SecurityError> {
        let mut header = Header::new(self.algorithm);
        header.kid = Some("access".to_string());

        let token = encode(&header, user_claims, &self.encoding_key)
            .map_err(|e| SecurityError::InvalidToken(format!("Failed to encode token: {}", e)))?;

        info!("Generated access token for user: {}", user_claims.sub);
        Ok(token)
    }

    /// Generate refresh token
    pub fn generate_refresh_token(&self, user_id: &str) -> Result<String, SecurityError> {
        let now = Utc::now();
        let exp = now + self.refresh_token_expiry;

        let claims = RefreshTokenClaims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            token_type: "refresh".to_string(),
        };

        let mut header = Header::new(self.algorithm);
        header.kid = Some("refresh".to_string());

        let token = encode(&header, &claims, &self.encoding_key).map_err(|e| {
            SecurityError::InvalidToken(format!("Failed to encode refresh token: {}", e))
        })?;

        info!("Generated refresh token for user: {}", user_id);
        Ok(token)
    }

    /// Validate and decode access token
    pub fn validate_access_token(&self, token: &str) -> Result<UserClaims, SecurityError> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        let token_data =
            decode::<UserClaims>(token, &self.decoding_key, &validation).map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        SecurityError::TokenExpired
                    }
                    _ => SecurityError::InvalidToken(format!("Token validation failed: {}", e)),
                }
            })?;

        let claims = token_data.claims;

        // Additional validation
        if claims.is_expired() {
            return Err(SecurityError::TokenExpired);
        }

        Ok(claims)
    }

    /// Validate and decode refresh token
    pub fn validate_refresh_token(&self, token: &str) -> Result<RefreshTokenClaims, SecurityError> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.validate_exp = true;

        let token_data = decode::<RefreshTokenClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => SecurityError::TokenExpired,
                _ => SecurityError::InvalidToken(format!("Refresh token validation failed: {}", e)),
            })?;

        let claims = token_data.claims;

        if claims.token_type != "refresh" {
            return Err(SecurityError::InvalidToken(
                "Invalid token type".to_string(),
            ));
        }

        Ok(claims)
    }

    /// Extract token from Authorization header
    pub fn extract_token_from_header(&self, auth_header: &str) -> Result<String, SecurityError> {
        if !auth_header.starts_with("Bearer ") {
            return Err(SecurityError::InvalidToken(
                "Authorization header must start with 'Bearer '".to_string(),
            ));
        }

        let token = auth_header.trim_start_matches("Bearer ").trim();
        if token.is_empty() {
            return Err(SecurityError::InvalidToken("Token is empty".to_string()));
        }

        Ok(token.to_string())
    }

    /// Create user claims for token generation
    pub fn create_user_claims(
        &self,
        user_id: String,
        email: String,
        roles: Vec<String>,
        permissions: Vec<String>,
    ) -> UserClaims {
        let now = Utc::now();
        let exp = now + self.access_token_expiry;

        UserClaims {
            sub: user_id,
            email,
            roles,
            permissions,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
        }
    }
}

/// Refresh token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: String,        // Subject (user ID)
    pub exp: i64,           // Expiration time
    pub iat: i64,           // Issued at
    pub jti: String,        // JWT ID
    pub iss: String,        // Issuer
    pub aud: String,        // Audience
    pub token_type: String, // Token type
}

/// Token blacklist manager for logout and revocation
pub struct TokenBlacklist {
    blacklisted_tokens: HashSet<String>,
}

impl TokenBlacklist {
    pub fn new() -> Self {
        Self {
            blacklisted_tokens: HashSet::new(),
        }
    }

    /// Add token to blacklist
    pub fn blacklist_token(&mut self, jti: &str) {
        self.blacklisted_tokens.insert(jti.to_string());
        info!("Token blacklisted: {}", jti);
    }

    /// Check if token is blacklisted
    pub fn is_blacklisted(&self, jti: &str) -> bool {
        self.blacklisted_tokens.contains(jti)
    }

    /// Remove expired tokens from blacklist
    pub fn cleanup_expired(&mut self, _current_timestamp: i64) {
        // In a real implementation, you'd need to store expiration times
        // and remove tokens that have expired
        info!("Cleaning up expired tokens from blacklist");
    }
}

/// Token pair for authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

impl TokenPair {
    pub fn new(access_token: String, refresh_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        }
    }
}

/// JWT utilities
pub struct JwtUtils;

impl JwtUtils {
    /// Decode token without validation (for inspection)
    pub fn decode_without_validation(token: &str) -> Result<UserClaims, SecurityError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.validate_aud = false;

        // Use a dummy key since we're not validating signature
        let dummy_key = DecodingKey::from_secret(b"dummy");

        let token_data = decode::<UserClaims>(token, &dummy_key, &validation)
            .map_err(|e| SecurityError::InvalidToken(format!("Failed to decode token: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Extract expiration time from token
    pub fn get_token_expiration(token: &str) -> Result<i64, SecurityError> {
        let claims = Self::decode_without_validation(token)?;
        Ok(claims.exp)
    }

    /// Check if token is expired
    pub fn is_token_expired(token: &str) -> Result<bool, SecurityError> {
        let exp = Self::get_token_expiration(token)?;
        Ok(Utc::now().timestamp() > exp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_manager_creation() {
        let manager = JwtManager::new(
            "this_is_a_very_long_secret_key_for_testing_purposes",
            1,
            7,
            "rwa-platform".to_string(),
            "rwa-users".to_string(),
        );
        assert!(manager.is_ok());

        // Test short secret
        let short_secret = JwtManager::new(
            "short",
            1,
            7,
            "rwa-platform".to_string(),
            "rwa-users".to_string(),
        );
        assert!(short_secret.is_err());
    }

    #[test]
    fn test_token_generation_and_validation() {
        let manager = JwtManager::new(
            "this_is_a_very_long_secret_key_for_testing_purposes",
            1,
            7,
            "rwa-platform".to_string(),
            "rwa-users".to_string(),
        )
        .unwrap();

        let claims = manager.create_user_claims(
            "user123".to_string(),
            "user@example.com".to_string(),
            vec!["user".to_string()],
            vec!["read".to_string()],
        );

        let token = manager.generate_access_token(&claims).unwrap();
        let validated_claims = manager.validate_access_token(&token).unwrap();

        assert_eq!(claims.sub, validated_claims.sub);
        assert_eq!(claims.email, validated_claims.email);
    }

    #[test]
    fn test_refresh_token() {
        let manager = JwtManager::new(
            "this_is_a_very_long_secret_key_for_testing_purposes",
            1,
            7,
            "rwa-platform".to_string(),
            "rwa-users".to_string(),
        )
        .unwrap();

        let refresh_token = manager.generate_refresh_token("user123").unwrap();
        let claims = manager.validate_refresh_token(&refresh_token).unwrap();

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.token_type, "refresh");
    }

    #[test]
    fn test_token_extraction() {
        let manager = JwtManager::new(
            "this_is_a_very_long_secret_key_for_testing_purposes",
            1,
            7,
            "rwa-platform".to_string(),
            "rwa-users".to_string(),
        )
        .unwrap();

        let token = manager.extract_token_from_header("Bearer abc123").unwrap();
        assert_eq!(token, "abc123");

        let invalid = manager.extract_token_from_header("Invalid header");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_token_blacklist() {
        let mut blacklist = TokenBlacklist::new();

        assert!(!blacklist.is_blacklisted("token123"));

        blacklist.blacklist_token("token123");
        assert!(blacklist.is_blacklisted("token123"));
    }
}
