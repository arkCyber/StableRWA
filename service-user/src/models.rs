// =====================================================================================
// File: service-user/src/models.rs
// Description: Data models for User Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{UserError, UserResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

/// User model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub status: String,
    pub roles: Vec<String>,
    pub email_verified: bool,
    pub email_verification_token: Option<String>,
    pub password_reset_token: Option<String>,
    pub password_reset_expires_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User session model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
}

/// User repository for database operations
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new user
    pub async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        first_name: &str,
        last_name: &str,
        phone: Option<&str>,
        email_verification_token: Option<&str>,
    ) -> UserResult<User> {
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (
                id, email, password_hash, first_name, last_name, phone,
                status, roles, email_verified, email_verification_token,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
            user_id,
            email,
            password_hash,
            first_name,
            last_name,
            phone,
            "pending_verification",
            &vec!["user".to_string()],
            false,
            email_verification_token,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> UserResult<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1 AND status != 'deleted'",
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(&self, user_id: &Uuid) -> UserResult<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1 AND status != 'deleted'",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by email verification token
    pub async fn find_by_verification_token(&self, token: &str) -> UserResult<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email_verification_token = $1 AND status != 'deleted'",
            token
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by password reset token
    pub async fn find_by_reset_token(&self, token: &str) -> UserResult<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users 
            WHERE password_reset_token = $1 
            AND password_reset_expires_at > NOW() 
            AND status != 'deleted'
            "#,
            token
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Update user
    pub async fn update_user(&self, user: &User) -> UserResult<User> {
        let updated_user = sqlx::query_as!(
            User,
            r#"
            UPDATE users SET
                email = $2,
                password_hash = $3,
                first_name = $4,
                last_name = $5,
                phone = $6,
                status = $7,
                roles = $8,
                email_verified = $9,
                email_verification_token = $10,
                password_reset_token = $11,
                password_reset_expires_at = $12,
                last_login_at = $13,
                login_attempts = $14,
                locked_until = $15,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            user.id,
            user.email,
            user.password_hash,
            user.first_name,
            user.last_name,
            user.phone,
            user.status,
            &user.roles,
            user.email_verified,
            user.email_verification_token,
            user.password_reset_token,
            user.password_reset_expires_at,
            user.last_login_at,
            user.login_attempts,
            user.locked_until
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_user)
    }

    /// Update user profile
    pub async fn update_profile(
        &self,
        user_id: &Uuid,
        first_name: Option<&str>,
        last_name: Option<&str>,
        phone: Option<&str>,
    ) -> UserResult<User> {
        let mut user = self.find_by_id(user_id).await?
            .ok_or_else(|| UserError::UserNotFound(user_id.to_string()))?;

        if let Some(first_name) = first_name {
            user.first_name = first_name.to_string();
        }
        if let Some(last_name) = last_name {
            user.last_name = last_name.to_string();
        }
        if let Some(phone) = phone {
            user.phone = Some(phone.to_string());
        }

        self.update_user(&user).await
    }

    /// Update password
    pub async fn update_password(&self, user_id: &Uuid, password_hash: &str) -> UserResult<()> {
        sqlx::query!(
            r#"
            UPDATE users SET
                password_hash = $2,
                password_reset_token = NULL,
                password_reset_expires_at = NULL,
                updated_at = NOW()
            WHERE id = $1
            "#,
            user_id,
            password_hash
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Verify email
    pub async fn verify_email(&self, user_id: &Uuid) -> UserResult<()> {
        sqlx::query!(
            r#"
            UPDATE users SET
                email_verified = true,
                email_verification_token = NULL,
                status = CASE 
                    WHEN status = 'pending_verification' THEN 'active'
                    ELSE status
                END,
                updated_at = NOW()
            WHERE id = $1
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Set password reset token
    pub async fn set_password_reset_token(
        &self,
        user_id: &Uuid,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> UserResult<()> {
        sqlx::query!(
            r#"
            UPDATE users SET
                password_reset_token = $2,
                password_reset_expires_at = $3,
                updated_at = NOW()
            WHERE id = $1
            "#,
            user_id,
            token,
            expires_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update login attempts
    pub async fn update_login_attempts(&self, user_id: &Uuid, attempts: i32) -> UserResult<()> {
        sqlx::query!(
            "UPDATE users SET login_attempts = $2, updated_at = NOW() WHERE id = $1",
            user_id,
            attempts
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Lock user account
    pub async fn lock_account(&self, user_id: &Uuid, locked_until: DateTime<Utc>) -> UserResult<()> {
        sqlx::query!(
            "UPDATE users SET locked_until = $2, updated_at = NOW() WHERE id = $1",
            user_id,
            locked_until
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update last login
    pub async fn update_last_login(&self, user_id: &Uuid) -> UserResult<()> {
        sqlx::query!(
            r#"
            UPDATE users SET
                last_login_at = NOW(),
                login_attempts = 0,
                locked_until = NULL,
                updated_at = NOW()
            WHERE id = $1
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// List users with pagination
    pub async fn list_users(&self, page: i64, per_page: i64) -> UserResult<(Vec<User>, i64)> {
        let offset = (page - 1) * per_page;

        let users = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users 
            WHERE status != 'deleted'
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            per_page,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE status != 'deleted'"
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok((users, total))
    }

    /// Delete user (soft delete)
    pub async fn delete_user(&self, user_id: &Uuid) -> UserResult<()> {
        sqlx::query!(
            "UPDATE users SET status = 'deleted', updated_at = NOW() WHERE id = $1",
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Activate user
    pub async fn activate_user(&self, user_id: &Uuid) -> UserResult<User> {
        let user = sqlx::query_as!(
            User,
            "UPDATE users SET status = 'active', updated_at = NOW() WHERE id = $1 RETURNING *",
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /// Deactivate user
    pub async fn deactivate_user(&self, user_id: &Uuid) -> UserResult<User> {
        let user = sqlx::query_as!(
            User,
            "UPDATE users SET status = 'inactive', updated_at = NOW() WHERE id = $1 RETURNING *",
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    // Session management methods

    /// Create user session
    pub async fn create_session(
        &self,
        user_id: &Uuid,
        refresh_token_hash: &str,
        expires_at: DateTime<Utc>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> UserResult<UserSession> {
        let session_id = Uuid::new_v4();
        let now = Utc::now();

        let session = sqlx::query_as!(
            UserSession,
            r#"
            INSERT INTO user_sessions (
                id, user_id, refresh_token_hash, expires_at,
                created_at, last_used_at, ip_address, user_agent, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            session_id,
            user_id,
            refresh_token_hash,
            expires_at,
            now,
            now,
            ip_address,
            user_agent,
            true
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    /// Find session by refresh token hash
    pub async fn find_session_by_token(&self, token_hash: &str) -> UserResult<Option<UserSession>> {
        let session = sqlx::query_as!(
            UserSession,
            r#"
            SELECT * FROM user_sessions 
            WHERE refresh_token_hash = $1 
            AND expires_at > NOW() 
            AND is_active = true
            "#,
            token_hash
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    /// Update session last used
    pub async fn update_session_last_used(&self, session_id: &Uuid) -> UserResult<()> {
        sqlx::query!(
            "UPDATE user_sessions SET last_used_at = NOW() WHERE id = $1",
            session_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// List user sessions
    pub async fn list_user_sessions(&self, user_id: &Uuid) -> UserResult<Vec<UserSession>> {
        let sessions = sqlx::query_as!(
            UserSession,
            r#"
            SELECT * FROM user_sessions 
            WHERE user_id = $1 AND is_active = true
            ORDER BY last_used_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(sessions)
    }

    /// Revoke session
    pub async fn revoke_session(&self, session_id: &Uuid) -> UserResult<()> {
        sqlx::query!(
            "UPDATE user_sessions SET is_active = false WHERE id = $1",
            session_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Revoke all user sessions
    pub async fn revoke_all_user_sessions(&self, user_id: &Uuid) -> UserResult<i64> {
        let result = sqlx::query!(
            "UPDATE user_sessions SET is_active = false WHERE user_id = $1 AND is_active = true",
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> UserResult<i64> {
        let result = sqlx::query!(
            "DELETE FROM user_sessions WHERE expires_at < NOW() OR is_active = false"
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    // Note: These tests would require a test database setup
    // They are provided as examples of how to test the repository

    async fn create_test_user(repo: &UserRepository) -> User {
        repo.create_user(
            "test@example.com",
            "hashed_password",
            "Test",
            "User",
            Some("+1234567890"),
            Some("verification_token"),
        )
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_and_find_user(pool: PgPool) {
        let repo = UserRepository::new(pool);
        
        let user = create_test_user(&repo).await;
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.first_name, "Test");
        assert_eq!(user.last_name, "User");
        assert_eq!(user.status, "pending_verification");
        assert!(!user.email_verified);

        let found_user = repo.find_by_email("test@example.com").await.unwrap();
        assert!(found_user.is_some());
        assert_eq!(found_user.unwrap().id, user.id);
    }

    #[sqlx::test]
    async fn test_update_user_profile(pool: PgPool) {
        let repo = UserRepository::new(pool);
        let user = create_test_user(&repo).await;

        let updated_user = repo.update_profile(
            &user.id,
            Some("Updated"),
            Some("Name"),
            Some("+9876543210"),
        ).await.unwrap();

        assert_eq!(updated_user.first_name, "Updated");
        assert_eq!(updated_user.last_name, "Name");
        assert_eq!(updated_user.phone, Some("+9876543210".to_string()));
    }

    #[sqlx::test]
    async fn test_verify_email(pool: PgPool) {
        let repo = UserRepository::new(pool);
        let user = create_test_user(&repo).await;

        repo.verify_email(&user.id).await.unwrap();

        let verified_user = repo.find_by_id(&user.id).await.unwrap().unwrap();
        assert!(verified_user.email_verified);
        assert_eq!(verified_user.status, "active");
        assert!(verified_user.email_verification_token.is_none());
    }

    #[sqlx::test]
    async fn test_session_management(pool: PgPool) {
        let repo = UserRepository::new(pool);
        let user = create_test_user(&repo).await;

        let expires_at = Utc::now() + chrono::Duration::hours(24);
        let session = repo.create_session(
            &user.id,
            "token_hash",
            expires_at,
            Some("127.0.0.1"),
            Some("Test Agent"),
        ).await.unwrap();

        assert_eq!(session.user_id, user.id);
        assert!(session.is_active);

        let found_session = repo.find_session_by_token("token_hash").await.unwrap();
        assert!(found_session.is_some());
        assert_eq!(found_session.unwrap().id, session.id);

        let sessions = repo.list_user_sessions(&user.id).await.unwrap();
        assert_eq!(sessions.len(), 1);

        repo.revoke_session(&session.id).await.unwrap();
        let revoked_session = repo.find_session_by_token("token_hash").await.unwrap();
        assert!(revoked_session.is_none());
    }
}
