// =====================================================================================
// File: service-user/src/service.rs
// Description: User service business logic implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{UserError, UserRepository, User, UserProfile, UserPreferences};
use async_trait::async_trait;
use core_security::{AuthenticationService, JwtManager, PasswordHasher, UserClaims};
use core_utils::{validation::RwaValidate, helpers::ApiResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// User service trait
#[async_trait]
pub trait UserService: Send + Sync {
    /// Register a new user
    async fn register_user(&self, request: RegisterUserRequest) -> Result<UserResponse, UserError>;
    
    /// Authenticate user and return tokens
    async fn authenticate_user(&self, request: LoginRequest) -> Result<AuthResponse, UserError>;
    
    /// Refresh access token
    async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse, UserError>;
    
    /// Get user by ID
    async fn get_user(&self, user_id: &str) -> Result<Option<UserResponse>, UserError>;
    
    /// Update user profile
    async fn update_profile(&self, user_id: &str, request: UpdateProfileRequest) -> Result<UserResponse, UserError>;
    
    /// Change user password
    async fn change_password(&self, user_id: &str, request: ChangePasswordRequest) -> Result<(), UserError>;
    
    /// Verify user email
    async fn verify_email(&self, user_id: &str, verification_code: &str) -> Result<(), UserError>;
    
    /// Request password reset
    async fn request_password_reset(&self, email: &str) -> Result<(), UserError>;
    
    /// Reset password with token
    async fn reset_password(&self, token: &str, new_password: &str) -> Result<(), UserError>;
    
    /// Deactivate user account
    async fn deactivate_user(&self, user_id: &str) -> Result<(), UserError>;
    
    /// Get user preferences
    async fn get_preferences(&self, user_id: &str) -> Result<UserPreferences, UserError>;
    
    /// Update user preferences
    async fn update_preferences(&self, user_id: &str, preferences: UserPreferences) -> Result<(), UserError>;
}

/// User service implementation
pub struct UserServiceImpl {
    repository: Arc<dyn UserRepository>,
    auth_service: Arc<dyn AuthenticationService>,
    jwt_manager: Arc<JwtManager>,
    password_hasher: Arc<PasswordHasher>,
}

impl UserServiceImpl {
    pub fn new(
        repository: Arc<dyn UserRepository>,
        auth_service: Arc<dyn AuthenticationService>,
        jwt_manager: Arc<JwtManager>,
        password_hasher: Arc<PasswordHasher>,
    ) -> Self {
        Self {
            repository,
            auth_service,
            jwt_manager,
            password_hasher,
        }
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn register_user(&self, request: RegisterUserRequest) -> Result<UserResponse, UserError> {
        info!(
            email = %request.email,
            first_name = %request.first_name,
            last_name = %request.last_name,
            "Registering new user"
        );

        // Validate input
        RwaValidate::user_registration(
            &request.email,
            &request.password,
            &request.first_name,
            &request.last_name,
        ).map_err(|e| UserError::ValidationError(e.to_string()))?;

        // Check if user already exists
        if let Some(_) = self.repository.find_by_email(&request.email).await? {
            return Err(UserError::UserAlreadyExists(request.email));
        }

        // Hash password
        let password_hash = self.password_hasher.hash_password(&request.password)
            .map_err(|e| UserError::InternalError(format!("Password hashing failed: {}", e)))?;

        // Create user
        let user = User {
            id: Uuid::new_v4().to_string(),
            email: request.email.clone(),
            password_hash,
            first_name: request.first_name,
            last_name: request.last_name,
            is_active: true,
            is_verified: false,
            email_verification_token: Some(Uuid::new_v4().to_string()),
            password_reset_token: None,
            password_reset_expires: None,
            last_login: None,
            failed_login_attempts: 0,
            locked_until: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let created_user = self.repository.create(&user).await?;

        // Create default profile
        let profile = UserProfile {
            user_id: created_user.id.clone(),
            avatar_url: None,
            bio: None,
            phone: request.phone,
            date_of_birth: None,
            country: None,
            timezone: Some("UTC".to_string()),
            language: Some("en".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.repository.create_profile(&profile).await?;

        // Create default preferences
        let preferences = UserPreferences::default();
        self.repository.update_preferences(&created_user.id, &preferences).await?;

        info!(
            user_id = %created_user.id,
            email = %created_user.email,
            "User registered successfully"
        );

        Ok(UserResponse::from_user_and_profile(created_user, Some(profile)))
    }

    async fn authenticate_user(&self, request: LoginRequest) -> Result<AuthResponse, UserError> {
        info!(email = %request.email, "Authenticating user");

        // Find user by email
        let user = self.repository.find_by_email(&request.email).await?
            .ok_or_else(|| UserError::InvalidCredentials)?;

        // Check if account is locked
        if let Some(locked_until) = user.locked_until {
            if locked_until > chrono::Utc::now() {
                return Err(UserError::AccountLocked);
            }
        }

        // Verify password
        let password_valid = self.password_hasher.verify_password(&request.password, &user.password_hash)
            .map_err(|e| UserError::InternalError(format!("Password verification failed: {}", e)))?;

        if !password_valid {
            // Increment failed login attempts
            self.repository.increment_failed_login_attempts(&user.id).await?;
            
            warn!(
                user_id = %user.id,
                email = %user.email,
                "Invalid password attempt"
            );
            
            return Err(UserError::InvalidCredentials);
        }

        // Check if user is active
        if !user.is_active {
            return Err(UserError::AccountDeactivated);
        }

        // Reset failed login attempts and update last login
        self.repository.reset_failed_login_attempts(&user.id).await?;
        self.repository.update_last_login(&user.id).await?;

        // Generate tokens
        let claims = UserClaims {
            sub: user.id.clone(),
            email: user.email.clone(),
            roles: vec!["user".to_string()],
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        };

        let access_token = self.jwt_manager.generate_access_token(&claims)
            .map_err(|e| UserError::InternalError(format!("Token generation failed: {}", e)))?;

        let refresh_token = self.jwt_manager.generate_refresh_token(&user.id)
            .map_err(|e| UserError::InternalError(format!("Refresh token generation failed: {}", e)))?;

        info!(
            user_id = %user.id,
            email = %user.email,
            "User authenticated successfully"
        );

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: UserResponse::from_user(user),
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse, UserError> {
        debug!("Refreshing access token");

        // Validate refresh token
        let user_id = self.jwt_manager.validate_refresh_token(refresh_token)
            .map_err(|e| UserError::InvalidToken(e.to_string()))?;

        // Get user
        let user = self.repository.find_by_id(&user_id).await?
            .ok_or_else(|| UserError::UserNotFound(user_id.clone()))?;

        // Check if user is active
        if !user.is_active {
            return Err(UserError::AccountDeactivated);
        }

        // Generate new tokens
        let claims = UserClaims {
            sub: user.id.clone(),
            email: user.email.clone(),
            roles: vec!["user".to_string()],
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        };

        let access_token = self.jwt_manager.generate_access_token(&claims)
            .map_err(|e| UserError::InternalError(format!("Token generation failed: {}", e)))?;

        let new_refresh_token = self.jwt_manager.generate_refresh_token(&user.id)
            .map_err(|e| UserError::InternalError(format!("Refresh token generation failed: {}", e)))?;

        Ok(AuthResponse {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: UserResponse::from_user(user),
        })
    }

    async fn get_user(&self, user_id: &str) -> Result<Option<UserResponse>, UserError> {
        debug!(user_id = %user_id, "Getting user");

        let user = self.repository.find_by_id(user_id).await?;
        if let Some(user) = user {
            let profile = self.repository.get_profile(user_id).await?;
            Ok(Some(UserResponse::from_user_and_profile(user, profile)))
        } else {
            Ok(None)
        }
    }

    async fn update_profile(&self, user_id: &str, request: UpdateProfileRequest) -> Result<UserResponse, UserError> {
        info!(user_id = %user_id, "Updating user profile");

        // Validate input
        if let Some(ref phone) = request.phone {
            core_security::validation::Validate::phone(phone, "phone")
                .map_err(|e| UserError::ValidationError(e.to_string()))?;
        }

        // Get existing profile or create new one
        let mut profile = self.repository.get_profile(user_id).await?
            .unwrap_or_else(|| UserProfile {
                user_id: user_id.to_string(),
                avatar_url: None,
                bio: None,
                phone: None,
                date_of_birth: None,
                country: None,
                timezone: Some("UTC".to_string()),
                language: Some("en".to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            });

        // Update profile fields
        if let Some(avatar_url) = request.avatar_url {
            profile.avatar_url = Some(avatar_url);
        }
        if let Some(bio) = request.bio {
            profile.bio = Some(bio);
        }
        if let Some(phone) = request.phone {
            profile.phone = Some(phone);
        }
        if let Some(date_of_birth) = request.date_of_birth {
            profile.date_of_birth = Some(date_of_birth);
        }
        if let Some(country) = request.country {
            profile.country = Some(country);
        }
        if let Some(timezone) = request.timezone {
            profile.timezone = Some(timezone);
        }
        if let Some(language) = request.language {
            profile.language = Some(language);
        }

        profile.updated_at = chrono::Utc::now();

        // Update profile in repository
        self.repository.update_profile(&profile).await?;

        // Get updated user
        let user = self.repository.find_by_id(user_id).await?
            .ok_or_else(|| UserError::UserNotFound(user_id.to_string()))?;

        Ok(UserResponse::from_user_and_profile(user, Some(profile)))
    }

    async fn change_password(&self, user_id: &str, request: ChangePasswordRequest) -> Result<(), UserError> {
        info!(user_id = %user_id, "Changing user password");

        // Validate new password
        core_security::validation::Validate::password_strength(&request.new_password, "new_password")
            .map_err(|e| UserError::ValidationError(e.to_string()))?;

        // Get user
        let user = self.repository.find_by_id(user_id).await?
            .ok_or_else(|| UserError::UserNotFound(user_id.to_string()))?;

        // Verify current password
        let password_valid = self.password_hasher.verify_password(&request.current_password, &user.password_hash)
            .map_err(|e| UserError::InternalError(format!("Password verification failed: {}", e)))?;

        if !password_valid {
            return Err(UserError::InvalidCredentials);
        }

        // Hash new password
        let new_password_hash = self.password_hasher.hash_password(&request.new_password)
            .map_err(|e| UserError::InternalError(format!("Password hashing failed: {}", e)))?;

        // Update password
        self.repository.update_password(user_id, &new_password_hash).await?;

        info!(user_id = %user_id, "Password changed successfully");
        Ok(())
    }

    async fn verify_email(&self, user_id: &str, verification_code: &str) -> Result<(), UserError> {
        info!(user_id = %user_id, "Verifying user email");

        let user = self.repository.find_by_id(user_id).await?
            .ok_or_else(|| UserError::UserNotFound(user_id.to_string()))?;

        if user.is_verified {
            return Err(UserError::EmailAlreadyVerified);
        }

        if let Some(ref token) = user.email_verification_token {
            if token == verification_code {
                self.repository.verify_email(user_id).await?;
                info!(user_id = %user_id, "Email verified successfully");
                Ok(())
            } else {
                Err(UserError::InvalidVerificationCode)
            }
        } else {
            Err(UserError::NoVerificationCodeFound)
        }
    }

    async fn request_password_reset(&self, email: &str) -> Result<(), UserError> {
        info!(email = %email, "Requesting password reset");

        if let Some(user) = self.repository.find_by_email(email).await? {
            let reset_token = Uuid::new_v4().to_string();
            let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
            
            self.repository.set_password_reset_token(&user.id, &reset_token, expires_at).await?;
            
            // In a real implementation, you would send an email here
            info!(user_id = %user.id, "Password reset token generated");
        }
        
        // Always return success to prevent email enumeration
        Ok(())
    }

    async fn reset_password(&self, token: &str, new_password: &str) -> Result<(), UserError> {
        info!("Resetting password with token");

        // Validate new password
        core_security::validation::Validate::password_strength(new_password, "new_password")
            .map_err(|e| UserError::ValidationError(e.to_string()))?;

        // Find user by reset token
        let user = self.repository.find_by_password_reset_token(token).await?
            .ok_or_else(|| UserError::InvalidResetToken)?;

        // Check if token is expired
        if let Some(expires_at) = user.password_reset_expires {
            if expires_at < chrono::Utc::now() {
                return Err(UserError::ResetTokenExpired);
            }
        } else {
            return Err(UserError::InvalidResetToken);
        }

        // Hash new password
        let password_hash = self.password_hasher.hash_password(new_password)
            .map_err(|e| UserError::InternalError(format!("Password hashing failed: {}", e)))?;

        // Update password and clear reset token
        self.repository.update_password(&user.id, &password_hash).await?;
        self.repository.clear_password_reset_token(&user.id).await?;

        info!(user_id = %user.id, "Password reset successfully");
        Ok(())
    }

    async fn deactivate_user(&self, user_id: &str) -> Result<(), UserError> {
        info!(user_id = %user_id, "Deactivating user");

        self.repository.deactivate_user(user_id).await?;
        
        info!(user_id = %user_id, "User deactivated successfully");
        Ok(())
    }

    async fn get_preferences(&self, user_id: &str) -> Result<UserPreferences, UserError> {
        debug!(user_id = %user_id, "Getting user preferences");

        self.repository.get_preferences(user_id).await?
            .ok_or_else(|| UserError::UserNotFound(user_id.to_string()))
    }

    async fn update_preferences(&self, user_id: &str, preferences: UserPreferences) -> Result<(), UserError> {
        info!(user_id = %user_id, "Updating user preferences");

        self.repository.update_preferences(user_id, &preferences).await?;
        
        info!(user_id = %user_id, "User preferences updated successfully");
        Ok(())
    }
}

/// Request/Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterUserRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub country: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
    pub is_verified: bool,
    pub profile: Option<UserProfile>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserResponse {
    pub fn from_user(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            is_active: user.is_active,
            is_verified: user.is_verified,
            profile: None,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }

    pub fn from_user_and_profile(user: User, profile: Option<UserProfile>) -> Self {
        Self {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            is_active: user.is_active,
            is_verified: user.is_verified,
            profile,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserResponse,
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_utils::fixtures::UserFixture;
    use std::collections::HashMap;

    // Mock implementations for testing
    struct MockUserRepository {
        users: std::sync::Mutex<HashMap<String, User>>,
        profiles: std::sync::Mutex<HashMap<String, UserProfile>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: std::sync::Mutex::new(HashMap::new()),
                profiles: std::sync::Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn create(&self, user: &User) -> Result<User, UserError> {
            let mut users = self.users.lock().unwrap();
            users.insert(user.id.clone(), user.clone());
            Ok(user.clone())
        }

        async fn find_by_id(&self, id: &str) -> Result<Option<User>, UserError> {
            let users = self.users.lock().unwrap();
            Ok(users.get(id).cloned())
        }

        async fn find_by_email(&self, email: &str) -> Result<Option<User>, UserError> {
            let users = self.users.lock().unwrap();
            Ok(users.values().find(|u| u.email == email).cloned())
        }

        async fn update(&self, user: &User) -> Result<User, UserError> {
            let mut users = self.users.lock().unwrap();
            users.insert(user.id.clone(), user.clone());
            Ok(user.clone())
        }

        async fn delete(&self, id: &str) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            users.remove(id);
            Ok(())
        }

        async fn create_profile(&self, profile: &UserProfile) -> Result<UserProfile, UserError> {
            let mut profiles = self.profiles.lock().unwrap();
            profiles.insert(profile.user_id.clone(), profile.clone());
            Ok(profile.clone())
        }

        async fn get_profile(&self, user_id: &str) -> Result<Option<UserProfile>, UserError> {
            let profiles = self.profiles.lock().unwrap();
            Ok(profiles.get(user_id).cloned())
        }

        async fn update_profile(&self, profile: &UserProfile) -> Result<UserProfile, UserError> {
            let mut profiles = self.profiles.lock().unwrap();
            profiles.insert(profile.user_id.clone(), profile.clone());
            Ok(profile.clone())
        }

        async fn verify_email(&self, user_id: &str) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.get_mut(user_id) {
                user.is_verified = true;
                user.email_verification_token = None;
            }
            Ok(())
        }

        async fn update_password(&self, user_id: &str, password_hash: &str) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.get_mut(user_id) {
                user.password_hash = password_hash.to_string();
            }
            Ok(())
        }

        async fn increment_failed_login_attempts(&self, user_id: &str) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.get_mut(user_id) {
                user.failed_login_attempts += 1;
            }
            Ok(())
        }

        async fn reset_failed_login_attempts(&self, user_id: &str) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.get_mut(user_id) {
                user.failed_login_attempts = 0;
            }
            Ok(())
        }

        async fn update_last_login(&self, user_id: &str) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.get_mut(user_id) {
                user.last_login = Some(chrono::Utc::now());
            }
            Ok(())
        }

        async fn set_password_reset_token(&self, user_id: &str, token: &str, expires_at: chrono::DateTime<chrono::Utc>) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.get_mut(user_id) {
                user.password_reset_token = Some(token.to_string());
                user.password_reset_expires = Some(expires_at);
            }
            Ok(())
        }

        async fn find_by_password_reset_token(&self, token: &str) -> Result<Option<User>, UserError> {
            let users = self.users.lock().unwrap();
            Ok(users.values().find(|u| u.password_reset_token.as_ref() == Some(token)).cloned())
        }

        async fn clear_password_reset_token(&self, user_id: &str) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.get_mut(user_id) {
                user.password_reset_token = None;
                user.password_reset_expires = None;
            }
            Ok(())
        }

        async fn deactivate_user(&self, user_id: &str) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.get_mut(user_id) {
                user.is_active = false;
            }
            Ok(())
        }

        async fn get_preferences(&self, _user_id: &str) -> Result<Option<UserPreferences>, UserError> {
            Ok(Some(UserPreferences::default()))
        }

        async fn update_preferences(&self, _user_id: &str, _preferences: &UserPreferences) -> Result<(), UserError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_user_registration() {
        let repository = Arc::new(MockUserRepository::new());
        let auth_service = Arc::new(core_security::InMemoryAuthenticationService::new());
        let jwt_manager = Arc::new(JwtManager::new(
            "test_secret_key_32_characters_long",
            1,
            7,
            "test".to_string(),
            "test".to_string(),
        ).unwrap());
        let password_hasher = Arc::new(PasswordHasher::new());

        let service = UserServiceImpl::new(repository, auth_service, jwt_manager, password_hasher);

        let request = RegisterUserRequest {
            email: "test@example.com".to_string(),
            password: "StrongPassword123!".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            phone: Some("+1234567890".to_string()),
        };

        let result = service.register_user(request).await;
        assert!(result.is_ok());

        let user_response = result.unwrap();
        assert_eq!(user_response.email, "test@example.com");
        assert_eq!(user_response.first_name, "John");
        assert_eq!(user_response.last_name, "Doe");
        assert!(!user_response.is_verified);
    }
}
