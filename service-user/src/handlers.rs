// =====================================================================================
// File: service-user/src/handlers.rs
// Description: HTTP handlers for User Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{models::*, service::UserService, AppState, UserError, UserResult};
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

/// User registration request
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
}

/// User login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Token refresh request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Email verification request
#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

/// Forgot password request
#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

/// Reset password request
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

/// Update user profile request
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
}

/// Authentication response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

/// User response
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub status: String,
    pub roles: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            phone: user.phone,
            status: user.status,
            roles: user.roles,
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login_at: user.last_login_at,
        }
    }
}

/// List users response
#[derive(Debug, Serialize)]
pub struct ListUsersResponse {
    pub users: Vec<UserResponse>,
    pub pagination: PaginationResponse,
}

/// Pagination response
#[derive(Debug, Serialize)]
pub struct PaginationResponse {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

/// Register a new user
pub async fn register_user(
    state: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> ActixResult<HttpResponse> {
    info!("User registration attempt for email: {}", req.email);

    match state.user_service.register_user(
        &req.email,
        &req.password,
        &req.first_name,
        &req.last_name,
        req.phone.as_deref(),
    ).await {
        Ok(auth_response) => {
            info!("User registered successfully: {}", req.email);
            Ok(HttpResponse::Created().json(auth_response))
        }
        Err(UserError::EmailAlreadyExists(_)) => {
            warn!("Registration failed - email already exists: {}", req.email);
            Ok(HttpResponse::Conflict().json(serde_json::json!({
                "error": "email_already_exists",
                "message": "A user with this email already exists"
            })))
        }
        Err(UserError::WeakPassword(msg)) => {
            warn!("Registration failed - weak password: {}", req.email);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "weak_password",
                "message": msg
            })))
        }
        Err(UserError::InvalidEmailFormat(_)) => {
            warn!("Registration failed - invalid email format: {}", req.email);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_email",
                "message": "Invalid email format"
            })))
        }
        Err(e) => {
            error!("Registration failed for {}: {}", req.email, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "registration_failed",
                "message": "Failed to register user"
            })))
        }
    }
}

/// Login user
pub async fn login_user(
    state: web::Data<AppState>,
    req: web::Json<LoginRequest>,
) -> ActixResult<HttpResponse> {
    info!("Login attempt for email: {}", req.email);

    match state.user_service.login_user(&req.email, &req.password).await {
        Ok(auth_response) => {
            info!("User logged in successfully: {}", req.email);
            Ok(HttpResponse::Ok().json(auth_response))
        }
        Err(UserError::InvalidCredentials) => {
            warn!("Login failed - invalid credentials: {}", req.email);
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "invalid_credentials",
                "message": "Invalid email or password"
            })))
        }
        Err(UserError::AccountNotActivated) => {
            warn!("Login failed - account not activated: {}", req.email);
            Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "account_not_activated",
                "message": "Please verify your email address"
            })))
        }
        Err(UserError::AccountSuspended) => {
            warn!("Login failed - account suspended: {}", req.email);
            Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "account_suspended",
                "message": "Your account has been suspended"
            })))
        }
        Err(UserError::TooManyLoginAttempts) => {
            warn!("Login failed - too many attempts: {}", req.email);
            Ok(HttpResponse::TooManyRequests().json(serde_json::json!({
                "error": "too_many_attempts",
                "message": "Too many login attempts. Please try again later"
            })))
        }
        Err(e) => {
            error!("Login failed for {}: {}", req.email, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "login_failed",
                "message": "Failed to login"
            })))
        }
    }
}

/// Logout user
pub async fn logout_user(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    if let Some(token) = extract_token(&req) {
        match state.user_service.logout_user(&token).await {
            Ok(_) => {
                info!("User logged out successfully");
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Logged out successfully"
                })))
            }
            Err(e) => {
                error!("Logout failed: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "logout_failed",
                    "message": "Failed to logout"
                })))
            }
        }
    } else {
        Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "missing_token",
            "message": "Authorization token required"
        })))
    }
}

/// Refresh access token
pub async fn refresh_token(
    state: web::Data<AppState>,
    req: web::Json<RefreshTokenRequest>,
) -> ActixResult<HttpResponse> {
    match state.user_service.refresh_token(&req.refresh_token).await {
        Ok(auth_response) => {
            info!("Token refreshed successfully");
            Ok(HttpResponse::Ok().json(auth_response))
        }
        Err(UserError::TokenExpired) => {
            warn!("Token refresh failed - token expired");
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "token_expired",
                "message": "Refresh token has expired"
            })))
        }
        Err(UserError::InvalidVerificationToken) => {
            warn!("Token refresh failed - invalid token");
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "invalid_token",
                "message": "Invalid refresh token"
            })))
        }
        Err(e) => {
            error!("Token refresh failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "refresh_failed",
                "message": "Failed to refresh token"
            })))
        }
    }
}

/// Verify email address
pub async fn verify_email(
    state: web::Data<AppState>,
    req: web::Json<VerifyEmailRequest>,
) -> ActixResult<HttpResponse> {
    match state.user_service.verify_email(&req.token).await {
        Ok(_) => {
            info!("Email verified successfully");
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Email verified successfully"
            })))
        }
        Err(UserError::InvalidVerificationToken) => {
            warn!("Email verification failed - invalid token");
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_token",
                "message": "Invalid verification token"
            })))
        }
        Err(UserError::TokenExpired) => {
            warn!("Email verification failed - token expired");
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "token_expired",
                "message": "Verification token has expired"
            })))
        }
        Err(e) => {
            error!("Email verification failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "verification_failed",
                "message": "Failed to verify email"
            })))
        }
    }
}

/// Request password reset
pub async fn forgot_password(
    state: web::Data<AppState>,
    req: web::Json<ForgotPasswordRequest>,
) -> ActixResult<HttpResponse> {
    match state.user_service.request_password_reset(&req.email).await {
        Ok(_) => {
            info!("Password reset requested for: {}", req.email);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Password reset email sent"
            })))
        }
        Err(UserError::UserNotFound(_)) => {
            // Don't reveal if user exists or not
            info!("Password reset requested for non-existent user: {}", req.email);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Password reset email sent"
            })))
        }
        Err(e) => {
            error!("Password reset request failed for {}: {}", req.email, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "reset_request_failed",
                "message": "Failed to process password reset request"
            })))
        }
    }
}

/// Reset password
pub async fn reset_password(
    state: web::Data<AppState>,
    req: web::Json<ResetPasswordRequest>,
) -> ActixResult<HttpResponse> {
    match state.user_service.reset_password(&req.token, &req.new_password).await {
        Ok(_) => {
            info!("Password reset successfully");
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Password reset successfully"
            })))
        }
        Err(UserError::InvalidVerificationToken) => {
            warn!("Password reset failed - invalid token");
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_token",
                "message": "Invalid reset token"
            })))
        }
        Err(UserError::TokenExpired) => {
            warn!("Password reset failed - token expired");
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "token_expired",
                "message": "Reset token has expired"
            })))
        }
        Err(UserError::WeakPassword(msg)) => {
            warn!("Password reset failed - weak password");
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "weak_password",
                "message": msg
            })))
        }
        Err(e) => {
            error!("Password reset failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "reset_failed",
                "message": "Failed to reset password"
            })))
        }
    }
}

/// Get user profile
pub async fn get_user_profile(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    if let Some(user_id) = extract_user_id(&req) {
        match state.user_service.get_user_by_id(&user_id).await {
            Ok(user) => {
                Ok(HttpResponse::Ok().json(UserResponse::from(user)))
            }
            Err(UserError::UserNotFound(_)) => {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "user_not_found",
                    "message": "User not found"
                })))
            }
            Err(e) => {
                error!("Failed to get user profile: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "profile_fetch_failed",
                    "message": "Failed to fetch user profile"
                })))
            }
        }
    } else {
        Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "unauthorized",
            "message": "Authentication required"
        })))
    }
}

/// Update user profile
pub async fn update_user_profile(
    state: web::Data<AppState>,
    req: HttpRequest,
    update_req: web::Json<UpdateProfileRequest>,
) -> ActixResult<HttpResponse> {
    if let Some(user_id) = extract_user_id(&req) {
        match state.user_service.update_user_profile(
            &user_id,
            update_req.first_name.as_deref(),
            update_req.last_name.as_deref(),
            update_req.phone.as_deref(),
        ).await {
            Ok(user) => {
                info!("User profile updated: {}", user_id);
                Ok(HttpResponse::Ok().json(UserResponse::from(user)))
            }
            Err(UserError::UserNotFound(_)) => {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "user_not_found",
                    "message": "User not found"
                })))
            }
            Err(UserError::ValidationError(msg)) => {
                Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "validation_error",
                    "message": msg
                })))
            }
            Err(e) => {
                error!("Failed to update user profile: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "update_failed",
                    "message": "Failed to update user profile"
                })))
            }
        }
    } else {
        Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "unauthorized",
            "message": "Authentication required"
        })))
    }
}

/// List users (admin only)
pub async fn list_users(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<PaginationQuery>,
) -> ActixResult<HttpResponse> {
    // Check admin permissions
    if !has_admin_permission(&req) {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "permission_denied",
            "message": "Admin access required"
        })));
    }

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100); // Max 100 per page

    match state.user_service.list_users(page, per_page).await {
        Ok((users, total)) => {
            let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
            let total_pages = (total as f64 / per_page as f64).ceil() as i64;

            Ok(HttpResponse::Ok().json(ListUsersResponse {
                users: user_responses,
                pagination: PaginationResponse {
                    page,
                    per_page,
                    total,
                    total_pages,
                },
            }))
        }
        Err(e) => {
            error!("Failed to list users: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "list_failed",
                "message": "Failed to list users"
            })))
        }
    }
}

/// Get user by ID (admin only)
pub async fn get_user(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    // Check admin permissions
    if !has_admin_permission(&req) {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "permission_denied",
            "message": "Admin access required"
        })));
    }

    let user_id = path.into_inner();
    let uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_user_id",
                "message": "Invalid user ID format"
            })));
        }
    };

    match state.user_service.get_user_by_id(&uuid).await {
        Ok(user) => {
            Ok(HttpResponse::Ok().json(UserResponse::from(user)))
        }
        Err(UserError::UserNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "user_not_found",
                "message": "User not found"
            })))
        }
        Err(e) => {
            error!("Failed to get user: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "fetch_failed",
                "message": "Failed to fetch user"
            })))
        }
    }
}

/// Create user (admin only)
pub async fn create_user(
    state: web::Data<AppState>,
    req: HttpRequest,
    create_req: web::Json<RegisterRequest>,
) -> ActixResult<HttpResponse> {
    // Check admin permissions
    if !has_admin_permission(&req) {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "permission_denied",
            "message": "Admin access required"
        })));
    }

    // Admin can create users without going through normal registration flow
    match state.user_service.create_user_admin(
        &create_req.email,
        &create_req.password,
        &create_req.first_name,
        &create_req.last_name,
        create_req.phone.as_deref(),
    ).await {
        Ok(user) => {
            info!("User created by admin: {}", create_req.email);
            Ok(HttpResponse::Created().json(UserResponse::from(user)))
        }
        Err(UserError::EmailAlreadyExists(_)) => {
            Ok(HttpResponse::Conflict().json(serde_json::json!({
                "error": "email_already_exists",
                "message": "A user with this email already exists"
            })))
        }
        Err(UserError::WeakPassword(msg)) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "weak_password",
                "message": msg
            })))
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "creation_failed",
                "message": "Failed to create user"
            })))
        }
    }
}

/// Update user (admin only)
pub async fn update_user(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    update_req: web::Json<UpdateProfileRequest>,
) -> ActixResult<HttpResponse> {
    // Check admin permissions
    if !has_admin_permission(&req) {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "permission_denied",
            "message": "Admin access required"
        })));
    }

    let user_id = path.into_inner();
    let uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_user_id",
                "message": "Invalid user ID format"
            })));
        }
    };

    match state.user_service.update_user_profile(
        &uuid,
        update_req.first_name.as_deref(),
        update_req.last_name.as_deref(),
        update_req.phone.as_deref(),
    ).await {
        Ok(user) => {
            info!("User updated by admin: {}", uuid);
            Ok(HttpResponse::Ok().json(UserResponse::from(user)))
        }
        Err(UserError::UserNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "user_not_found",
                "message": "User not found"
            })))
        }
        Err(e) => {
            error!("Failed to update user: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "update_failed",
                "message": "Failed to update user"
            })))
        }
    }
}

/// Delete user (admin only)
pub async fn delete_user(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    // Check admin permissions
    if !has_admin_permission(&req) {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "permission_denied",
            "message": "Admin access required"
        })));
    }

    let user_id = path.into_inner();
    let uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_user_id",
                "message": "Invalid user ID format"
            })));
        }
    };

    match state.user_service.delete_user(&uuid).await {
        Ok(_) => {
            info!("User deleted by admin: {}", uuid);
            Ok(HttpResponse::NoContent().finish())
        }
        Err(UserError::UserNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "user_not_found",
                "message": "User not found"
            })))
        }
        Err(e) => {
            error!("Failed to delete user: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "deletion_failed",
                "message": "Failed to delete user"
            })))
        }
    }
}

/// Activate user (admin only)
pub async fn activate_user(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    // Check admin permissions
    if !has_admin_permission(&req) {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "permission_denied",
            "message": "Admin access required"
        })));
    }

    let user_id = path.into_inner();
    let uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_user_id",
                "message": "Invalid user ID format"
            })));
        }
    };

    match state.user_service.activate_user(&uuid).await {
        Ok(user) => {
            info!("User activated by admin: {}", uuid);
            Ok(HttpResponse::Ok().json(UserResponse::from(user)))
        }
        Err(UserError::UserNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "user_not_found",
                "message": "User not found"
            })))
        }
        Err(e) => {
            error!("Failed to activate user: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "activation_failed",
                "message": "Failed to activate user"
            })))
        }
    }
}

/// Deactivate user (admin only)
pub async fn deactivate_user(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    // Check admin permissions
    if !has_admin_permission(&req) {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "permission_denied",
            "message": "Admin access required"
        })));
    }

    let user_id = path.into_inner();
    let uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_user_id",
                "message": "Invalid user ID format"
            })));
        }
    };

    match state.user_service.deactivate_user(&uuid).await {
        Ok(user) => {
            info!("User deactivated by admin: {}", uuid);
            Ok(HttpResponse::Ok().json(UserResponse::from(user)))
        }
        Err(UserError::UserNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "user_not_found",
                "message": "User not found"
            })))
        }
        Err(e) => {
            error!("Failed to deactivate user: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "deactivation_failed",
                "message": "Failed to deactivate user"
            })))
        }
    }
}

/// List user sessions
pub async fn list_user_sessions(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    if let Some(user_id) = extract_user_id(&req) {
        match state.user_service.list_user_sessions(&user_id).await {
            Ok(sessions) => {
                Ok(HttpResponse::Ok().json(sessions))
            }
            Err(e) => {
                error!("Failed to list user sessions: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "sessions_fetch_failed",
                    "message": "Failed to fetch user sessions"
                })))
            }
        }
    } else {
        Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "unauthorized",
            "message": "Authentication required"
        })))
    }
}

/// Revoke session
pub async fn revoke_session(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    if let Some(user_id) = extract_user_id(&req) {
        let session_id = path.into_inner();
        let uuid = match Uuid::parse_str(&session_id) {
            Ok(uuid) => uuid,
            Err(_) => {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "invalid_session_id",
                    "message": "Invalid session ID format"
                })));
            }
        };

        match state.user_service.revoke_session(&user_id, &uuid).await {
            Ok(_) => {
                info!("Session revoked: {}", uuid);
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Session revoked successfully"
                })))
            }
            Err(UserError::SessionNotFound) => {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "session_not_found",
                    "message": "Session not found"
                })))
            }
            Err(e) => {
                error!("Failed to revoke session: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "revoke_failed",
                    "message": "Failed to revoke session"
                })))
            }
        }
    } else {
        Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "unauthorized",
            "message": "Authentication required"
        })))
    }
}

/// Revoke all sessions
pub async fn revoke_all_sessions(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    if let Some(user_id) = extract_user_id(&req) {
        match state.user_service.revoke_all_sessions(&user_id).await {
            Ok(count) => {
                info!("All sessions revoked for user: {}, count: {}", user_id, count);
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "All sessions revoked successfully",
                    "revoked_count": count
                })))
            }
            Err(e) => {
                error!("Failed to revoke all sessions: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "revoke_all_failed",
                    "message": "Failed to revoke all sessions"
                })))
            }
        }
    } else {
        Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "unauthorized",
            "message": "Authentication required"
        })))
    }
}

/// Health check endpoint
pub async fn health_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "user-service",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Readiness check endpoint
pub async fn readiness_check(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    // Check database connectivity
    let db_healthy = state.database.health_check().await.is_ok();
    
    let status = if db_healthy { "ready" } else { "not_ready" };
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": status,
        "service": "user-service",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "checks": {
            "database": db_healthy
        }
    })))
}

/// Liveness check endpoint
pub async fn liveness_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "service": "user-service",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Metrics endpoint
pub async fn metrics_endpoint(state: web::Data<AppState>) -> ActixResult<String> {
    let metrics = state.metrics.export_prometheus_metrics().await;
    Ok(metrics)
}

// Helper functions

/// Extract JWT token from Authorization header
fn extract_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
}

/// Extract user ID from JWT token in request
fn extract_user_id(req: &HttpRequest) -> Option<Uuid> {
    // This would typically extract user ID from validated JWT claims
    // For now, return None - this should be implemented with proper JWT validation
    req.extensions()
        .get::<Uuid>()
        .copied()
}

/// Check if user has admin permissions
fn has_admin_permission(req: &HttpRequest) -> bool {
    // This would typically check user roles from validated JWT claims
    // For now, return false - this should be implemented with proper role checking
    req.extensions()
        .get::<Vec<String>>()
        .map(|roles| roles.contains(&"admin".to_string()))
        .unwrap_or(false)
}

/// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
