// =====================================================================================
// File: core-institutional/src/custody_test.rs
// Description: Comprehensive tests for custody service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::custody::*;
    use crate::types::*;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use uuid::Uuid;
    use tokio_test;

    /// Mock custody service for testing
    struct MockCustodyService {
        accounts: std::sync::Arc<std::sync::Mutex<HashMap<Uuid, CustodyAccount>>>,
        transactions: std::sync::Arc<std::sync::Mutex<HashMap<Uuid, CustodyTransactionResult>>>,
    }

    impl MockCustodyService {
        fn new() -> Self {
            Self {
                accounts: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
                transactions: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl CustodyService for MockCustodyService {
        async fn create_account(&self, account: CustodyAccount) -> InstitutionalResult<CustodyAccount> {
            let mut accounts = self.accounts.lock().unwrap();
            accounts.insert(account.id, account.clone());
            Ok(account)
        }

        async fn get_account(&self, account_id: Uuid) -> InstitutionalResult<Option<CustodyAccount>> {
            let accounts = self.accounts.lock().unwrap();
            Ok(accounts.get(&account_id).cloned())
        }

        async fn get_institution_accounts(&self, institution_id: Uuid) -> InstitutionalResult<Vec<CustodyAccount>> {
            let accounts = self.accounts.lock().unwrap();
            let filtered: Vec<CustodyAccount> = accounts
                .values()
                .filter(|account| account.institution_id == institution_id)
                .cloned()
                .collect();
            Ok(filtered)
        }

        async fn update_account(&self, account: CustodyAccount) -> InstitutionalResult<CustodyAccount> {
            let mut accounts = self.accounts.lock().unwrap();
            accounts.insert(account.id, account.clone());
            Ok(account)
        }

        async fn get_balances(&self, account_id: Uuid) -> InstitutionalResult<HashMap<String, AssetBalance>> {
            let accounts = self.accounts.lock().unwrap();
            if let Some(account) = accounts.get(&account_id) {
                Ok(account.balances.clone())
            } else {
                Err(InstitutionalError::not_found("CustodyAccount", account_id.to_string()))
            }
        }

        async fn submit_transaction(&self, request: CustodyTransactionRequest) -> InstitutionalResult<CustodyTransactionResult> {
            let result = CustodyTransactionResult {
                request_id: request.id,
                transaction_id: Some(Uuid::new_v4().to_string()),
                status: TransactionStatus::Pending,
                signatures_collected: 0,
                signatures_required: request.required_signatures,
                approvals: vec![],
                executed_at: None,
                error_message: None,
            };

            let mut transactions = self.transactions.lock().unwrap();
            transactions.insert(request.id, result.clone());
            Ok(result)
        }

        async fn approve_transaction(
            &self,
            request_id: Uuid,
            signer_id: Uuid,
            signature: String,
        ) -> InstitutionalResult<CustodyTransactionResult> {
            let mut transactions = self.transactions.lock().unwrap();
            if let Some(mut result) = transactions.get(&request_id).cloned() {
                result.signatures_collected += 1;
                result.approvals.push(TransactionApproval {
                    signer_id,
                    signature,
                    approved_at: Utc::now(),
                    ip_address: "127.0.0.1".to_string(),
                    user_agent: "test".to_string(),
                });

                if result.signatures_collected >= result.signatures_required {
                    result.status = TransactionStatus::Completed;
                    result.executed_at = Some(Utc::now());
                }

                transactions.insert(request_id, result.clone());
                Ok(result)
            } else {
                Err(InstitutionalError::not_found("Transaction", request_id.to_string()))
            }
        }

        async fn get_transaction_status(&self, request_id: Uuid) -> InstitutionalResult<Option<CustodyTransactionResult>> {
            let transactions = self.transactions.lock().unwrap();
            Ok(transactions.get(&request_id).cloned())
        }

        async fn get_transaction_history(
            &self,
            _account_id: Uuid,
            _limit: Option<usize>,
            _offset: Option<usize>,
        ) -> InstitutionalResult<Vec<CustodyTransactionResult>> {
            let transactions = self.transactions.lock().unwrap();
            Ok(transactions.values().cloned().collect())
        }

        async fn add_authorized_signer(&self, _account_id: Uuid, _signer: AuthorizedSigner) -> InstitutionalResult<()> {
            Ok(())
        }

        async fn remove_authorized_signer(&self, _account_id: Uuid, _signer_id: Uuid) -> InstitutionalResult<()> {
            Ok(())
        }

        async fn generate_custody_report(
            &self,
            account_id: Uuid,
            report_type: CustodyReportType,
            _start_date: chrono::DateTime<Utc>,
            _end_date: chrono::DateTime<Utc>,
        ) -> InstitutionalResult<CustodyReport> {
            Ok(CustodyReport {
                id: Uuid::new_v4(),
                account_id,
                report_type,
                report_data: serde_json::json!({}),
                generated_at: Utc::now(),
                generated_by: Uuid::new_v4(),
                file_url: None,
            })
        }

        async fn health_check(&self) -> InstitutionalResult<CustodyHealthStatus> {
            Ok(CustodyHealthStatus {
                status: "healthy".to_string(),
                total_accounts: 1,
                active_accounts: 1,
                total_assets_under_custody: Decimal::new(1000000000, 2),
                cold_storage_percentage: Decimal::new(95, 2),
                pending_transactions: 0,
                failed_transactions_24h: 0,
                insurance_coverage_ratio: Decimal::new(100, 2),
                last_reconciliation: Utc::now(),
                last_check: Utc::now(),
            })
        }
    }

    fn create_test_custody_account() -> CustodyAccount {
        let mut balances = HashMap::new();
        balances.insert("BTC".to_string(), AssetBalance {
            asset: "BTC".to_string(),
            total_balance: Decimal::new(10, 8), // 0.1 BTC
            available_balance: Decimal::new(8, 8), // 0.08 BTC
            pending_balance: Decimal::new(1, 8), // 0.01 BTC
            blocked_balance: Decimal::new(1, 8), // 0.01 BTC
            cold_storage_balance: Decimal::new(9, 8), // 0.09 BTC
            hot_wallet_balance: Decimal::new(1, 8), // 0.01 BTC
            last_updated: Utc::now(),
        });

        CustodyAccount {
            id: Uuid::new_v4(),
            institution_id: Uuid::new_v4(),
            account_number: "CUST001".to_string(),
            account_name: "Test Custody Account".to_string(),
            account_type: CustodyAccountType::Segregated,
            base_currency: "USD".to_string(),
            custodian: "Test Custodian".to_string(),
            sub_custodians: vec![],
            segregation_type: SegregationType::FullySegregated,
            insurance_policy: Some(InsurancePolicy {
                policy_number: "INS001".to_string(),
                insurer: "Test Insurance Co.".to_string(),
                coverage_amount: Decimal::new(100000000, 2), // $1M
                deductible: Decimal::new(10000, 2), // $100
                policy_type: InsurancePolicyType::Comprehensive,
                effective_date: Utc::now(),
                expiry_date: Utc::now() + chrono::Duration::days(365),
                covered_assets: vec!["BTC".to_string(), "ETH".to_string()],
            }),
            authorized_signers: vec![
                AuthorizedSigner {
                    id: Uuid::new_v4(),
                    name: "John Doe".to_string(),
                    title: "CEO".to_string(),
                    email: "john@example.com".to_string(),
                    public_key: "test_public_key".to_string(),
                    signing_authority: SigningAuthority::Executive,
                    is_active: true,
                    created_at: Utc::now(),
                },
            ],
            balances,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
        }
    }

    #[tokio::test]
    async fn test_create_custody_account() {
        let service = MockCustodyService::new();
        let account = create_test_custody_account();
        let account_id = account.id;

        let result = service.create_account(account).await;
        assert!(result.is_ok());

        let retrieved = service.get_account(account_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, account_id);
    }

    #[tokio::test]
    async fn test_custody_transaction_workflow() {
        let service = MockCustodyService::new();
        let account = create_test_custody_account();
        let account_id = account.id;
        
        // Create account first
        service.create_account(account).await.unwrap();

        // Create transaction request
        let transaction_request = CustodyTransactionRequest {
            id: Uuid::new_v4(),
            account_id,
            transaction_type: CustodyTransactionType::Withdrawal,
            asset: "BTC".to_string(),
            amount: Decimal::new(1, 8), // 0.01 BTC
            destination: Some("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string()),
            purpose: "Test withdrawal".to_string(),
            requested_by: Uuid::new_v4(),
            approvers: vec![Uuid::new_v4()],
            required_signatures: 2,
            deadline: Some(Utc::now() + chrono::Duration::hours(24)),
            created_at: Utc::now(),
        };

        let request_id = transaction_request.id;

        // Submit transaction
        let result = service.submit_transaction(transaction_request).await.unwrap();
        assert_eq!(result.status, TransactionStatus::Pending);
        assert_eq!(result.signatures_collected, 0);
        assert_eq!(result.signatures_required, 2);

        // First approval
        let signer1 = Uuid::new_v4();
        let result = service.approve_transaction(
            request_id,
            signer1,
            "signature1".to_string(),
        ).await.unwrap();
        assert_eq!(result.signatures_collected, 1);
        assert_eq!(result.status, TransactionStatus::Pending);

        // Second approval (should complete transaction)
        let signer2 = Uuid::new_v4();
        let result = service.approve_transaction(
            request_id,
            signer2,
            "signature2".to_string(),
        ).await.unwrap();
        assert_eq!(result.signatures_collected, 2);
        assert_eq!(result.status, TransactionStatus::Completed);
        assert!(result.executed_at.is_some());
    }

    #[tokio::test]
    async fn test_get_balances() {
        let service = MockCustodyService::new();
        let account = create_test_custody_account();
        let account_id = account.id;

        service.create_account(account).await.unwrap();

        let balances = service.get_balances(account_id).await.unwrap();
        assert!(balances.contains_key("BTC"));
        
        let btc_balance = &balances["BTC"];
        assert_eq!(btc_balance.total_balance, Decimal::new(10, 8));
        assert_eq!(btc_balance.available_balance, Decimal::new(8, 8));
    }

    #[tokio::test]
    async fn test_signing_authority_limits() {
        assert_eq!(SigningAuthority::Limited.max_amount(), Some(Decimal::new(10000000, 2)));
        assert_eq!(SigningAuthority::Standard.max_amount(), Some(Decimal::new(100000000, 2)));
        assert_eq!(SigningAuthority::Senior.max_amount(), Some(Decimal::new(1000000000, 2)));
        assert_eq!(SigningAuthority::Executive.max_amount(), None);
    }

    #[tokio::test]
    async fn test_custody_report_generation() {
        let service = MockCustodyService::new();
        let account = create_test_custody_account();
        let account_id = account.id;

        service.create_account(account).await.unwrap();

        let report = service.generate_custody_report(
            account_id,
            CustodyReportType::PositionReport,
            Utc::now() - chrono::Duration::days(30),
            Utc::now(),
        ).await.unwrap();

        assert_eq!(report.account_id, account_id);
        assert_eq!(report.report_type, CustodyReportType::PositionReport);
        assert!(report.generated_at <= Utc::now());
    }

    #[tokio::test]
    async fn test_health_check() {
        let service = MockCustodyService::new();
        let health = service.health_check().await.unwrap();

        assert_eq!(health.status, "healthy");
        assert_eq!(health.total_accounts, 1);
        assert_eq!(health.active_accounts, 1);
        assert_eq!(health.cold_storage_percentage, Decimal::new(95, 2));
    }

    #[tokio::test]
    async fn test_custody_config_validation() {
        let config = CustodyConfig::default();
        
        assert_eq!(config.multisig_threshold, 3);
        assert_eq!(config.high_value_signers, 5);
        assert!(config.enable_hsm);
        assert!(config.enable_monitoring);
        assert_eq!(config.audit_retention_days, 2555); // 7 years
    }

    #[tokio::test]
    async fn test_asset_balance_calculations() {
        let balance = AssetBalance {
            asset: "BTC".to_string(),
            total_balance: Decimal::new(100000000, 8), // 1 BTC
            available_balance: Decimal::new(80000000, 8), // 0.8 BTC
            pending_balance: Decimal::new(10000000, 8), // 0.1 BTC
            blocked_balance: Decimal::new(10000000, 8), // 0.1 BTC
            cold_storage_balance: Decimal::new(95000000, 8), // 0.95 BTC
            hot_wallet_balance: Decimal::new(5000000, 8), // 0.05 BTC
            last_updated: Utc::now(),
        };

        // Verify balance consistency
        assert_eq!(
            balance.available_balance + balance.pending_balance + balance.blocked_balance,
            balance.total_balance
        );
        
        assert_eq!(
            balance.cold_storage_balance + balance.hot_wallet_balance,
            balance.total_balance
        );
    }

    #[tokio::test]
    async fn test_error_handling() {
        let service = MockCustodyService::new();
        let non_existent_id = Uuid::new_v4();

        // Test getting non-existent account
        let result = service.get_account(non_existent_id).await.unwrap();
        assert!(result.is_none());

        // Test getting balances for non-existent account
        let result = service.get_balances(non_existent_id).await;
        assert!(result.is_err());
        
        if let Err(error) = result {
            assert!(matches!(error, InstitutionalError::NotFound { .. }));
        }
    }
}
