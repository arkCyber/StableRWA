// =====================================================================================
// File: service-payment/src/models.rs
// Description: Data models for Payment Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{PaymentError, PaymentResult};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

/// Payment model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub status: String,
    pub payment_method_id: Option<Uuid>,
    pub payment_method_type: String,
    pub provider: String,
    pub provider_payment_id: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
}

/// Payment method model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentMethod {
    pub id: Uuid,
    pub user_id: Uuid,
    pub method_type: String,
    pub provider: String,
    pub provider_payment_method_id: String,
    pub is_default: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payment refund model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentRefund {
    pub id: Uuid,
    pub payment_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub status: String,
    pub provider_refund_id: Option<String>,
    pub reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Payment status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentStatus {
    pub status: String,
    pub provider_status: Option<String>,
    pub last_updated: DateTime<Utc>,
    pub details: Option<serde_json::Value>,
}

/// Payment repository for database operations
pub struct PaymentRepository {
    pool: PgPool,
}

impl PaymentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new payment
    pub async fn create_payment(
        &self,
        user_id: &Uuid,
        amount: Decimal,
        currency: &str,
        payment_method_id: Option<&Uuid>,
        payment_method_type: &str,
        provider: &str,
        description: Option<&str>,
        metadata: Option<&serde_json::Value>,
    ) -> PaymentResult<Payment> {
        let payment_id = Uuid::new_v4();
        let now = Utc::now();

        let payment = sqlx::query_as!(
            Payment,
            r#"
            INSERT INTO payments (
                id, user_id, amount, currency, status, payment_method_id,
                payment_method_type, provider, description, metadata,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
            payment_id,
            user_id,
            amount,
            currency,
            "pending",
            payment_method_id,
            payment_method_type,
            provider,
            description,
            metadata,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(payment)
    }

    /// Find payment by ID
    pub async fn find_by_id(&self, payment_id: &Uuid) -> PaymentResult<Option<Payment>> {
        let payment = sqlx::query_as!(
            Payment,
            "SELECT * FROM payments WHERE id = $1",
            payment_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(payment)
    }

    /// Find payment by ID and user ID
    pub async fn find_by_id_and_user(
        &self,
        payment_id: &Uuid,
        user_id: &Uuid,
    ) -> PaymentResult<Option<Payment>> {
        let payment = sqlx::query_as!(
            Payment,
            "SELECT * FROM payments WHERE id = $1 AND user_id = $2",
            payment_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(payment)
    }

    /// Find payment by provider payment ID
    pub async fn find_by_provider_id(&self, provider_payment_id: &str) -> PaymentResult<Option<Payment>> {
        let payment = sqlx::query_as!(
            Payment,
            "SELECT * FROM payments WHERE provider_payment_id = $1",
            provider_payment_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(payment)
    }

    /// Update payment
    pub async fn update_payment(&self, payment: &Payment) -> PaymentResult<Payment> {
        let updated_payment = sqlx::query_as!(
            Payment,
            r#"
            UPDATE payments SET
                amount = $2,
                currency = $3,
                status = $4,
                payment_method_id = $5,
                payment_method_type = $6,
                provider = $7,
                provider_payment_id = $8,
                description = $9,
                metadata = $10,
                updated_at = NOW(),
                completed_at = $11,
                failed_at = $12,
                failure_reason = $13
            WHERE id = $1
            RETURNING *
            "#,
            payment.id,
            payment.amount,
            payment.currency,
            payment.status,
            payment.payment_method_id,
            payment.payment_method_type,
            payment.provider,
            payment.provider_payment_id,
            payment.description,
            payment.metadata,
            payment.completed_at,
            payment.failed_at,
            payment.failure_reason
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_payment)
    }

    /// Update payment status
    pub async fn update_payment_status(
        &self,
        payment_id: &Uuid,
        status: &str,
        provider_payment_id: Option<&str>,
        failure_reason: Option<&str>,
    ) -> PaymentResult<()> {
        let now = Utc::now();
        let (completed_at, failed_at) = match status {
            "completed" => (Some(now), None),
            "failed" | "cancelled" => (None, Some(now)),
            _ => (None, None),
        };

        sqlx::query!(
            r#"
            UPDATE payments SET
                status = $2,
                provider_payment_id = COALESCE($3, provider_payment_id),
                failure_reason = $4,
                completed_at = $5,
                failed_at = $6,
                updated_at = NOW()
            WHERE id = $1
            "#,
            payment_id,
            status,
            provider_payment_id,
            failure_reason,
            completed_at,
            failed_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// List user payments with pagination
    pub async fn list_user_payments(
        &self,
        user_id: &Uuid,
        page: i64,
        per_page: i64,
    ) -> PaymentResult<(Vec<Payment>, i64)> {
        let offset = (page - 1) * per_page;

        let payments = sqlx::query_as!(
            Payment,
            r#"
            SELECT * FROM payments 
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            per_page,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM payments WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok((payments, total))
    }

    /// List all payments with pagination (admin)
    pub async fn list_all_payments(&self, page: i64, per_page: i64) -> PaymentResult<(Vec<Payment>, i64)> {
        let offset = (page - 1) * per_page;

        let payments = sqlx::query_as!(
            Payment,
            r#"
            SELECT * FROM payments 
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            per_page,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar!("SELECT COUNT(*) FROM payments")
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

        Ok((payments, total))
    }

    // Payment method operations

    /// Create payment method
    pub async fn create_payment_method(
        &self,
        user_id: &Uuid,
        method_type: &str,
        provider: &str,
        provider_payment_method_id: &str,
        is_default: bool,
        metadata: Option<&serde_json::Value>,
    ) -> PaymentResult<PaymentMethod> {
        let method_id = Uuid::new_v4();
        let now = Utc::now();

        // If this is set as default, unset other default methods
        if is_default {
            sqlx::query!(
                "UPDATE payment_methods SET is_default = false WHERE user_id = $1",
                user_id
            )
            .execute(&self.pool)
            .await?;
        }

        let method = sqlx::query_as!(
            PaymentMethod,
            r#"
            INSERT INTO payment_methods (
                id, user_id, method_type, provider, provider_payment_method_id,
                is_default, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            method_id,
            user_id,
            method_type,
            provider,
            provider_payment_method_id,
            is_default,
            metadata,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(method)
    }

    /// Find payment method by ID
    pub async fn find_payment_method_by_id(&self, method_id: &Uuid) -> PaymentResult<Option<PaymentMethod>> {
        let method = sqlx::query_as!(
            PaymentMethod,
            "SELECT * FROM payment_methods WHERE id = $1",
            method_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(method)
    }

    /// Find payment method by ID and user ID
    pub async fn find_payment_method_by_id_and_user(
        &self,
        method_id: &Uuid,
        user_id: &Uuid,
    ) -> PaymentResult<Option<PaymentMethod>> {
        let method = sqlx::query_as!(
            PaymentMethod,
            "SELECT * FROM payment_methods WHERE id = $1 AND user_id = $2",
            method_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(method)
    }

    /// Find default payment method for user
    pub async fn find_default_payment_method(&self, user_id: &Uuid) -> PaymentResult<Option<PaymentMethod>> {
        let method = sqlx::query_as!(
            PaymentMethod,
            "SELECT * FROM payment_methods WHERE user_id = $1 AND is_default = true",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(method)
    }

    /// Update payment method
    pub async fn update_payment_method(&self, method: &PaymentMethod) -> PaymentResult<PaymentMethod> {
        // If this is set as default, unset other default methods
        if method.is_default {
            sqlx::query!(
                "UPDATE payment_methods SET is_default = false WHERE user_id = $1 AND id != $2",
                method.user_id,
                method.id
            )
            .execute(&self.pool)
            .await?;
        }

        let updated_method = sqlx::query_as!(
            PaymentMethod,
            r#"
            UPDATE payment_methods SET
                method_type = $2,
                provider = $3,
                provider_payment_method_id = $4,
                is_default = $5,
                metadata = $6,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            method.id,
            method.method_type,
            method.provider,
            method.provider_payment_method_id,
            method.is_default,
            method.metadata
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_method)
    }

    /// List user payment methods
    pub async fn list_user_payment_methods(
        &self,
        user_id: &Uuid,
        page: i64,
        per_page: i64,
    ) -> PaymentResult<(Vec<PaymentMethod>, i64)> {
        let offset = (page - 1) * per_page;

        let methods = sqlx::query_as!(
            PaymentMethod,
            r#"
            SELECT * FROM payment_methods 
            WHERE user_id = $1
            ORDER BY is_default DESC, created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            per_page,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM payment_methods WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok((methods, total))
    }

    /// Delete payment method
    pub async fn delete_payment_method(&self, method_id: &Uuid) -> PaymentResult<()> {
        sqlx::query!("DELETE FROM payment_methods WHERE id = $1", method_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Refund operations

    /// Create refund
    pub async fn create_refund(
        &self,
        payment_id: &Uuid,
        amount: Decimal,
        currency: &str,
        reason: Option<&str>,
        metadata: Option<&serde_json::Value>,
    ) -> PaymentResult<PaymentRefund> {
        let refund_id = Uuid::new_v4();
        let now = Utc::now();

        let refund = sqlx::query_as!(
            PaymentRefund,
            r#"
            INSERT INTO payment_refunds (
                id, payment_id, amount, currency, status, reason, metadata,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            refund_id,
            payment_id,
            amount,
            currency,
            "pending",
            reason,
            metadata,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(refund)
    }

    /// Find refund by ID
    pub async fn find_refund_by_id(&self, refund_id: &Uuid) -> PaymentResult<Option<PaymentRefund>> {
        let refund = sqlx::query_as!(
            PaymentRefund,
            "SELECT * FROM payment_refunds WHERE id = $1",
            refund_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(refund)
    }

    /// Update refund status
    pub async fn update_refund_status(
        &self,
        refund_id: &Uuid,
        status: &str,
        provider_refund_id: Option<&str>,
    ) -> PaymentResult<()> {
        let completed_at = if status == "completed" {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query!(
            r#"
            UPDATE payment_refunds SET
                status = $2,
                provider_refund_id = COALESCE($3, provider_refund_id),
                completed_at = $4,
                updated_at = NOW()
            WHERE id = $1
            "#,
            refund_id,
            status,
            provider_refund_id,
            completed_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// List payment refunds
    pub async fn list_payment_refunds(&self, payment_id: &Uuid) -> PaymentResult<Vec<PaymentRefund>> {
        let refunds = sqlx::query_as!(
            PaymentRefund,
            "SELECT * FROM payment_refunds WHERE payment_id = $1 ORDER BY created_at DESC",
            payment_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(refunds)
    }

    /// Get payment statistics
    pub async fn get_payment_statistics(&self, user_id: Option<&Uuid>) -> PaymentResult<serde_json::Value> {
        let stats = if let Some(user_id) = user_id {
            sqlx::query!(
                r#"
                SELECT 
                    COUNT(*) as total_payments,
                    COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_payments,
                    COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_payments,
                    COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_payments,
                    COALESCE(SUM(CASE WHEN status = 'completed' THEN amount ELSE 0 END), 0) as total_amount
                FROM payments 
                WHERE user_id = $1
                "#,
                user_id
            )
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query!(
                r#"
                SELECT 
                    COUNT(*) as total_payments,
                    COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_payments,
                    COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_payments,
                    COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_payments,
                    COALESCE(SUM(CASE WHEN status = 'completed' THEN amount ELSE 0 END), 0) as total_amount
                FROM payments
                "#
            )
            .fetch_one(&self.pool)
            .await?
        };

        Ok(serde_json::json!({
            "total_payments": stats.total_payments,
            "completed_payments": stats.completed_payments,
            "failed_payments": stats.failed_payments,
            "pending_payments": stats.pending_payments,
            "total_amount": stats.total_amount
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use sqlx::PgPool;

    async fn create_test_payment(repo: &PaymentRepository, user_id: &Uuid) -> Payment {
        repo.create_payment(
            user_id,
            dec!(100.00),
            "USD",
            None,
            "credit_card",
            "stripe",
            Some("Test payment"),
            None,
        )
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_and_find_payment(pool: PgPool) {
        let repo = PaymentRepository::new(pool);
        let user_id = Uuid::new_v4();
        
        let payment = create_test_payment(&repo, &user_id).await;
        assert_eq!(payment.amount, dec!(100.00));
        assert_eq!(payment.currency, "USD");
        assert_eq!(payment.status, "pending");

        let found_payment = repo.find_by_id(&payment.id).await.unwrap();
        assert!(found_payment.is_some());
        assert_eq!(found_payment.unwrap().id, payment.id);
    }

    #[sqlx::test]
    async fn test_update_payment_status(pool: PgPool) {
        let repo = PaymentRepository::new(pool);
        let user_id = Uuid::new_v4();
        let payment = create_test_payment(&repo, &user_id).await;

        repo.update_payment_status(
            &payment.id,
            "completed",
            Some("stripe_payment_123"),
            None,
        ).await.unwrap();

        let updated_payment = repo.find_by_id(&payment.id).await.unwrap().unwrap();
        assert_eq!(updated_payment.status, "completed");
        assert_eq!(updated_payment.provider_payment_id, Some("stripe_payment_123".to_string()));
        assert!(updated_payment.completed_at.is_some());
    }

    #[sqlx::test]
    async fn test_payment_method_operations(pool: PgPool) {
        let repo = PaymentRepository::new(pool);
        let user_id = Uuid::new_v4();

        let method = repo.create_payment_method(
            &user_id,
            "credit_card",
            "stripe",
            "pm_123456789",
            true,
            None,
        ).await.unwrap();

        assert_eq!(method.method_type, "credit_card");
        assert_eq!(method.provider, "stripe");
        assert!(method.is_default);

        let found_method = repo.find_payment_method_by_id(&method.id).await.unwrap();
        assert!(found_method.is_some());

        let default_method = repo.find_default_payment_method(&user_id).await.unwrap();
        assert!(default_method.is_some());
        assert_eq!(default_method.unwrap().id, method.id);

        let methods = repo.list_user_payment_methods(&user_id, 1, 10).await.unwrap();
        assert_eq!(methods.0.len(), 1);
        assert_eq!(methods.1, 1);
    }

    #[sqlx::test]
    async fn test_refund_operations(pool: PgPool) {
        let repo = PaymentRepository::new(pool);
        let user_id = Uuid::new_v4();
        let payment = create_test_payment(&repo, &user_id).await;

        let refund = repo.create_refund(
            &payment.id,
            dec!(50.00),
            "USD",
            Some("Customer request"),
            None,
        ).await.unwrap();

        assert_eq!(refund.payment_id, payment.id);
        assert_eq!(refund.amount, dec!(50.00));
        assert_eq!(refund.status, "pending");

        repo.update_refund_status(&refund.id, "completed", Some("re_123456789")).await.unwrap();

        let updated_refund = repo.find_refund_by_id(&refund.id).await.unwrap().unwrap();
        assert_eq!(updated_refund.status, "completed");
        assert!(updated_refund.completed_at.is_some());

        let refunds = repo.list_payment_refunds(&payment.id).await.unwrap();
        assert_eq!(refunds.len(), 1);
    }
}
