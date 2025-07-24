// =====================================================================================
// File: core-compliance/src/sanctions.rs
// Description: Sanctions screening and watchlist management
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{ComplianceError, ComplianceResult},
    types::{AmlCheck, AmlCheckType, AmlMatch, AmlResult, KycData, RiskLevel},
};

/// Sanctions list configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsConfig {
    /// Enabled sanctions lists
    pub enabled_lists: Vec<SanctionsList>,
    /// Update frequency in hours
    pub update_frequency_hours: u32,
    /// Fuzzy matching threshold (0.0 to 1.0)
    pub fuzzy_match_threshold: f64,
    /// Auto-update enabled
    pub auto_update: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u32,
}

impl Default for SanctionsConfig {
    fn default() -> Self {
        Self {
            enabled_lists: vec![
                SanctionsList::OFAC,
                SanctionsList::UN,
                SanctionsList::EU,
                SanctionsList::UK,
            ],
            update_frequency_hours: 24,
            fuzzy_match_threshold: 0.8,
            auto_update: true,
            cache_ttl_seconds: 3600,
        }
    }
}

/// Available sanctions lists
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SanctionsList {
    /// US Office of Foreign Assets Control
    OFAC,
    /// United Nations Security Council
    UN,
    /// European Union
    EU,
    /// United Kingdom
    UK,
    /// Financial Action Task Force
    FATF,
    /// Custom list
    Custom(u16),
}

impl SanctionsList {
    pub fn as_str(&self) -> &'static str {
        match self {
            SanctionsList::OFAC => "OFAC",
            SanctionsList::UN => "UN",
            SanctionsList::EU => "EU",
            SanctionsList::UK => "UK",
            SanctionsList::FATF => "FATF",
            SanctionsList::Custom(_) => "CUSTOM",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SanctionsList::OFAC => "US Office of Foreign Assets Control",
            SanctionsList::UN => "United Nations Security Council",
            SanctionsList::EU => "European Union Sanctions",
            SanctionsList::UK => "UK HM Treasury Sanctions",
            SanctionsList::FATF => "Financial Action Task Force",
            SanctionsList::Custom(_) => "Custom Sanctions List",
        }
    }
}

/// Sanctions entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsEntry {
    pub id: String,
    pub list_source: SanctionsList,
    pub entry_type: SanctionsEntryType,
    pub names: Vec<String>,
    pub aliases: Vec<String>,
    pub addresses: Vec<String>,
    pub nationalities: Vec<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub place_of_birth: Option<String>,
    pub sanctions_programs: Vec<String>,
    pub added_date: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub is_active: bool,
    pub metadata: HashMap<String, String>,
}

/// Type of sanctions entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SanctionsEntryType {
    Individual,
    Entity,
    Vessel,
    Aircraft,
    Address,
}

/// Sanctions match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsMatch {
    pub entry_id: String,
    pub list_source: SanctionsList,
    pub match_type: MatchType,
    pub confidence_score: f64,
    pub matched_field: String,
    pub matched_value: String,
    pub entry_names: Vec<String>,
    pub sanctions_programs: Vec<String>,
    pub match_details: MatchDetails,
}

/// Type of match found
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchType {
    ExactName,
    FuzzyName,
    Alias,
    Address,
    DateOfBirth,
    Nationality,
    Composite,
}

/// Detailed match information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchDetails {
    pub algorithm_used: String,
    pub similarity_score: f64,
    pub matched_tokens: Vec<String>,
    pub false_positive_indicators: Vec<String>,
}

/// Sanctions screening service
pub struct SanctionsService {
    config: SanctionsConfig,
    entries: HashMap<String, SanctionsEntry>,
    last_update: Option<DateTime<Utc>>,
}

impl SanctionsService {
    /// Create new sanctions service
    pub fn new(config: SanctionsConfig) -> Self {
        Self {
            config,
            entries: HashMap::new(),
            last_update: None,
        }
    }

    /// Load sanctions data from sources
    pub async fn load_sanctions_data(&mut self) -> ComplianceResult<()> {
        let enabled_lists = self.config.enabled_lists.clone();
        for list in enabled_lists {
            self.load_list_data(list).await?;
        }
        self.last_update = Some(Utc::now());
        Ok(())
    }

    /// Screen individual against sanctions lists
    pub async fn screen_individual(&self, kyc_data: &KycData) -> ComplianceResult<Vec<SanctionsMatch>> {
        let mut matches = Vec::new();

        // Screen name
        let name_matches = self.screen_name(&kyc_data.first_name, &kyc_data.last_name).await?;
        matches.extend(name_matches);

        // Screen nationality
        let nationality_matches = self.screen_nationality(&kyc_data.nationality).await?;
        matches.extend(nationality_matches);

        // Screen address
        let address_matches = self.screen_address(&kyc_data.address).await?;
        matches.extend(address_matches);

        // Screen date of birth
        let dob_matches = self.screen_date_of_birth(kyc_data.date_of_birth).await?;
        matches.extend(dob_matches);

        Ok(matches)
    }

    /// Check if sanctions data needs update
    pub fn needs_update(&self) -> bool {
        if !self.config.auto_update {
            return false;
        }

        match self.last_update {
            Some(last_update) => {
                let update_interval = chrono::Duration::hours(self.config.update_frequency_hours as i64);
                Utc::now() - last_update > update_interval
            }
            None => true,
        }
    }

    /// Get statistics about loaded sanctions data
    pub fn get_statistics(&self) -> SanctionsStatistics {
        let mut stats = SanctionsStatistics {
            total_entries: self.entries.len(),
            entries_by_list: HashMap::new(),
            entries_by_type: HashMap::new(),
            last_update: self.last_update,
        };

        for entry in self.entries.values() {
            *stats.entries_by_list.entry(entry.list_source).or_insert(0) += 1;
            *stats.entries_by_type.entry(entry.entry_type).or_insert(0) += 1;
        }

        stats
    }

    // Private helper methods
    async fn load_list_data(&mut self, list: SanctionsList) -> ComplianceResult<()> {
        // In a real implementation, this would fetch data from external sources
        // For now, we'll create some mock data
        match list {
            SanctionsList::OFAC => self.load_mock_ofac_data().await,
            SanctionsList::UN => self.load_mock_un_data().await,
            SanctionsList::EU => self.load_mock_eu_data().await,
            SanctionsList::UK => self.load_mock_uk_data().await,
            _ => Ok(()),
        }
    }

    async fn load_mock_ofac_data(&mut self) -> ComplianceResult<()> {
        // Mock OFAC data
        let entry = SanctionsEntry {
            id: "OFAC-001".to_string(),
            list_source: SanctionsList::OFAC,
            entry_type: SanctionsEntryType::Individual,
            names: vec!["John Doe".to_string()],
            aliases: vec!["Johnny Doe".to_string()],
            addresses: vec!["Unknown".to_string()],
            nationalities: vec!["XX".to_string()],
            date_of_birth: Some(chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
            place_of_birth: Some("Unknown".to_string()),
            sanctions_programs: vec!["Terrorism".to_string()],
            added_date: Utc::now(),
            last_updated: Utc::now(),
            is_active: true,
            metadata: HashMap::new(),
        };

        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    async fn load_mock_un_data(&mut self) -> ComplianceResult<()> {
        // Mock UN data - similar structure
        Ok(())
    }

    async fn load_mock_eu_data(&mut self) -> ComplianceResult<()> {
        // Mock EU data - similar structure
        Ok(())
    }

    async fn load_mock_uk_data(&mut self) -> ComplianceResult<()> {
        // Mock UK data - similar structure
        Ok(())
    }

    async fn screen_name(&self, first_name: &str, last_name: &str) -> ComplianceResult<Vec<SanctionsMatch>> {
        let full_name = format!("{} {}", first_name, last_name);
        let mut matches = Vec::new();

        for entry in self.entries.values() {
            if !entry.is_active {
                continue;
            }

            // Check exact matches
            for name in &entry.names {
                if self.is_exact_match(&full_name, name) {
                    matches.push(self.create_match(entry, MatchType::ExactName, 1.0, "name", &full_name));
                }
            }

            // Check fuzzy matches
            for name in &entry.names {
                let similarity = self.calculate_similarity(&full_name, name);
                if similarity >= self.config.fuzzy_match_threshold {
                    matches.push(self.create_match(entry, MatchType::FuzzyName, similarity, "name", &full_name));
                }
            }

            // Check aliases
            for alias in &entry.aliases {
                let similarity = self.calculate_similarity(&full_name, alias);
                if similarity >= self.config.fuzzy_match_threshold {
                    matches.push(self.create_match(entry, MatchType::Alias, similarity, "alias", &full_name));
                }
            }
        }

        Ok(matches)
    }

    async fn screen_nationality(&self, nationality: &str) -> ComplianceResult<Vec<SanctionsMatch>> {
        let mut matches = Vec::new();

        for entry in self.entries.values() {
            if !entry.is_active {
                continue;
            }

            for entry_nationality in &entry.nationalities {
                if entry_nationality.eq_ignore_ascii_case(nationality) {
                    matches.push(self.create_match(entry, MatchType::Nationality, 1.0, "nationality", nationality));
                }
            }
        }

        Ok(matches)
    }

    async fn screen_address(&self, address: &crate::types::Address) -> ComplianceResult<Vec<SanctionsMatch>> {
        let address_string = format!("{}, {}, {}", address.street, address.city, address.country);
        let mut matches = Vec::new();

        for entry in self.entries.values() {
            if !entry.is_active {
                continue;
            }

            for entry_address in &entry.addresses {
                let similarity = self.calculate_similarity(&address_string, entry_address);
                if similarity >= self.config.fuzzy_match_threshold {
                    matches.push(self.create_match(entry, MatchType::Address, similarity, "address", &address_string));
                }
            }
        }

        Ok(matches)
    }

    async fn screen_date_of_birth(&self, dob: chrono::NaiveDate) -> ComplianceResult<Vec<SanctionsMatch>> {
        let mut matches = Vec::new();

        for entry in self.entries.values() {
            if !entry.is_active {
                continue;
            }

            if let Some(entry_dob) = entry.date_of_birth {
                if entry_dob == dob {
                    matches.push(self.create_match(entry, MatchType::DateOfBirth, 1.0, "date_of_birth", &dob.to_string()));
                }
            }
        }

        Ok(matches)
    }

    fn is_exact_match(&self, text1: &str, text2: &str) -> bool {
        text1.trim().eq_ignore_ascii_case(text2.trim())
    }

    fn calculate_similarity(&self, text1: &str, text2: &str) -> f64 {
        // Simple similarity calculation (in real implementation, use proper fuzzy matching)
        let text1_lower = text1.to_lowercase();
        let text2_lower = text2.to_lowercase();

        if text1_lower == text2_lower {
            return 1.0;
        }

        // Simple token-based similarity
        let tokens1: std::collections::HashSet<&str> = text1_lower.split_whitespace().collect();
        let tokens2: std::collections::HashSet<&str> = text2_lower.split_whitespace().collect();

        let intersection = tokens1.intersection(&tokens2).count();
        let union = tokens1.union(&tokens2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    fn create_match(&self, entry: &SanctionsEntry, match_type: MatchType, confidence: f64, field: &str, value: &str) -> SanctionsMatch {
        SanctionsMatch {
            entry_id: entry.id.clone(),
            list_source: entry.list_source,
            match_type,
            confidence_score: confidence,
            matched_field: field.to_string(),
            matched_value: value.to_string(),
            entry_names: entry.names.clone(),
            sanctions_programs: entry.sanctions_programs.clone(),
            match_details: MatchDetails {
                algorithm_used: "token_similarity".to_string(),
                similarity_score: confidence,
                matched_tokens: vec![value.to_string()],
                false_positive_indicators: Vec::new(),
            },
        }
    }
}

/// Sanctions statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsStatistics {
    pub total_entries: usize,
    pub entries_by_list: HashMap<SanctionsList, usize>,
    pub entries_by_type: HashMap<SanctionsEntryType, usize>,
    pub last_update: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Address, DocumentType, IdentityDocument, VerificationStatus};

    fn create_test_kyc_data() -> KycData {
        KycData {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            date_of_birth: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            nationality: "US".to_string(),
            address: Address {
                street: "123 Main St".to_string(),
                city: "New York".to_string(),
                state_province: Some("NY".to_string()),
                postal_code: "10001".to_string(),
                country: "US".to_string(),
            },
            identity_document: IdentityDocument {
                document_type: DocumentType::Passport,
                document_number: "123456789".to_string(),
                issuing_country: "US".to_string(),
                issue_date: chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                expiry_date: Some(chrono::NaiveDate::from_ymd_opt(2030, 1, 1).unwrap()),
            },
            verification_status: VerificationStatus::NotStarted,
            verification_date: None,
            expiry_date: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_sanctions_config_default() {
        let config = SanctionsConfig::default();
        assert!(!config.enabled_lists.is_empty());
        assert!(config.enabled_lists.contains(&SanctionsList::OFAC));
        assert_eq!(config.update_frequency_hours, 24);
        assert_eq!(config.fuzzy_match_threshold, 0.8);
        assert!(config.auto_update);
    }

    #[test]
    fn test_sanctions_list_string_conversion() {
        assert_eq!(SanctionsList::OFAC.as_str(), "OFAC");
        assert_eq!(SanctionsList::UN.as_str(), "UN");
        assert_eq!(SanctionsList::EU.as_str(), "EU");
        assert_eq!(SanctionsList::UK.as_str(), "UK");
        assert_eq!(SanctionsList::Custom(123).as_str(), "CUSTOM");
    }

    #[test]
    fn test_sanctions_list_description() {
        assert_eq!(SanctionsList::OFAC.description(), "US Office of Foreign Assets Control");
        assert_eq!(SanctionsList::UN.description(), "United Nations Security Council");
        assert_eq!(SanctionsList::EU.description(), "European Union Sanctions");
        assert_eq!(SanctionsList::UK.description(), "UK HM Treasury Sanctions");
    }

    #[test]
    fn test_sanctions_service_creation() {
        let config = SanctionsConfig::default();
        let service = SanctionsService::new(config);
        assert_eq!(service.entries.len(), 0);
        assert!(service.last_update.is_none());
    }

    #[tokio::test]
    async fn test_sanctions_data_loading() {
        let config = SanctionsConfig::default();
        let mut service = SanctionsService::new(config);

        let result = service.load_sanctions_data().await;
        assert!(result.is_ok());
        assert!(service.last_update.is_some());
        assert!(!service.entries.is_empty());
    }

    #[test]
    fn test_needs_update() {
        let config = SanctionsConfig::default();
        let service = SanctionsService::new(config);

        // Should need update when no data loaded
        assert!(service.needs_update());
    }

    #[tokio::test]
    async fn test_individual_screening() {
        let config = SanctionsConfig::default();
        let mut service = SanctionsService::new(config);

        // Load test data
        service.load_sanctions_data().await.unwrap();

        let kyc_data = create_test_kyc_data();
        let matches = service.screen_individual(&kyc_data).await;
        assert!(matches.is_ok());

        let match_list = matches.unwrap();
        // Should find match for "John Doe" in mock OFAC data
        assert!(!match_list.is_empty());
    }

    #[tokio::test]
    async fn test_name_screening() {
        let config = SanctionsConfig::default();
        let mut service = SanctionsService::new(config);

        service.load_sanctions_data().await.unwrap();

        let matches = service.screen_name("John", "Doe").await;
        assert!(matches.is_ok());

        let match_list = matches.unwrap();
        assert!(!match_list.is_empty());

        let first_match = &match_list[0];
        assert_eq!(first_match.list_source, SanctionsList::OFAC);
        assert!(matches!(first_match.match_type, MatchType::ExactName | MatchType::FuzzyName));
    }

    #[tokio::test]
    async fn test_nationality_screening() {
        let config = SanctionsConfig::default();
        let mut service = SanctionsService::new(config);

        service.load_sanctions_data().await.unwrap();

        let matches = service.screen_nationality("XX").await;
        assert!(matches.is_ok());

        let match_list = matches.unwrap();
        if !match_list.is_empty() {
            let first_match = &match_list[0];
            assert_eq!(first_match.match_type, MatchType::Nationality);
        }
    }

    #[tokio::test]
    async fn test_address_screening() {
        let config = SanctionsConfig::default();
        let service = SanctionsService::new(config);

        let address = Address {
            street: "123 Test St".to_string(),
            city: "Test City".to_string(),
            state_province: Some("TS".to_string()),
            postal_code: "12345".to_string(),
            country: "US".to_string(),
        };

        let matches = service.screen_address(&address).await;
        assert!(matches.is_ok());
    }

    #[tokio::test]
    async fn test_date_of_birth_screening() {
        let config = SanctionsConfig::default();
        let mut service = SanctionsService::new(config);

        service.load_sanctions_data().await.unwrap();

        let dob = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let matches = service.screen_date_of_birth(dob).await;
        assert!(matches.is_ok());

        let match_list = matches.unwrap();
        if !match_list.is_empty() {
            let first_match = &match_list[0];
            assert_eq!(first_match.match_type, MatchType::DateOfBirth);
        }
    }

    #[test]
    fn test_similarity_calculation() {
        let config = SanctionsConfig::default();
        let service = SanctionsService::new(config);

        // Exact match
        assert_eq!(service.calculate_similarity("John Doe", "John Doe"), 1.0);

        // Partial match
        let similarity = service.calculate_similarity("John Doe", "John Smith");
        assert!(similarity > 0.0 && similarity < 1.0);

        // No match
        let no_similarity = service.calculate_similarity("John Doe", "Jane Wilson");
        assert!(no_similarity >= 0.0);
    }

    #[test]
    fn test_exact_match() {
        let config = SanctionsConfig::default();
        let service = SanctionsService::new(config);

        assert!(service.is_exact_match("John Doe", "john doe"));
        assert!(service.is_exact_match("  John Doe  ", "John Doe"));
        assert!(!service.is_exact_match("John Doe", "Jane Doe"));
    }

    #[tokio::test]
    async fn test_statistics() {
        let config = SanctionsConfig::default();
        let mut service = SanctionsService::new(config);

        service.load_sanctions_data().await.unwrap();

        let stats = service.get_statistics();
        assert!(stats.total_entries > 0);
        assert!(!stats.entries_by_list.is_empty());
        assert!(!stats.entries_by_type.is_empty());
        assert!(stats.last_update.is_some());
    }

    #[test]
    fn test_sanctions_entry_creation() {
        let entry = SanctionsEntry {
            id: "TEST-001".to_string(),
            list_source: SanctionsList::OFAC,
            entry_type: SanctionsEntryType::Individual,
            names: vec!["Test Person".to_string()],
            aliases: vec!["Test Alias".to_string()],
            addresses: vec!["Test Address".to_string()],
            nationalities: vec!["US".to_string()],
            date_of_birth: Some(chrono::NaiveDate::from_ymd_opt(1980, 1, 1).unwrap()),
            place_of_birth: Some("Test City".to_string()),
            sanctions_programs: vec!["Test Program".to_string()],
            added_date: Utc::now(),
            last_updated: Utc::now(),
            is_active: true,
            metadata: HashMap::new(),
        };

        assert_eq!(entry.id, "TEST-001");
        assert_eq!(entry.list_source, SanctionsList::OFAC);
        assert_eq!(entry.entry_type, SanctionsEntryType::Individual);
        assert!(entry.is_active);
    }

    #[test]
    fn test_sanctions_match_creation() {
        let config = SanctionsConfig::default();
        let service = SanctionsService::new(config);

        let entry = SanctionsEntry {
            id: "TEST-001".to_string(),
            list_source: SanctionsList::OFAC,
            entry_type: SanctionsEntryType::Individual,
            names: vec!["Test Person".to_string()],
            aliases: vec![],
            addresses: vec![],
            nationalities: vec![],
            date_of_birth: None,
            place_of_birth: None,
            sanctions_programs: vec!["Test Program".to_string()],
            added_date: Utc::now(),
            last_updated: Utc::now(),
            is_active: true,
            metadata: HashMap::new(),
        };

        let sanctions_match = service.create_match(&entry, MatchType::ExactName, 1.0, "name", "Test Person");

        assert_eq!(sanctions_match.entry_id, "TEST-001");
        assert_eq!(sanctions_match.list_source, SanctionsList::OFAC);
        assert_eq!(sanctions_match.match_type, MatchType::ExactName);
        assert_eq!(sanctions_match.confidence_score, 1.0);
        assert_eq!(sanctions_match.matched_field, "name");
        assert_eq!(sanctions_match.matched_value, "Test Person");
    }

    #[test]
    fn test_match_type_variants() {
        let match_types = vec![
            MatchType::ExactName,
            MatchType::FuzzyName,
            MatchType::Alias,
            MatchType::Address,
            MatchType::DateOfBirth,
            MatchType::Nationality,
            MatchType::Composite,
        ];

        for match_type in match_types {
            // Test serialization
            let json = serde_json::to_string(&match_type).expect("Failed to serialize");
            let deserialized: MatchType = serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(match_type, deserialized);
        }
    }

    #[test]
    fn test_sanctions_entry_type_variants() {
        let entry_types = vec![
            SanctionsEntryType::Individual,
            SanctionsEntryType::Entity,
            SanctionsEntryType::Vessel,
            SanctionsEntryType::Aircraft,
            SanctionsEntryType::Address,
        ];

        for entry_type in entry_types {
            // Test serialization
            let json = serde_json::to_string(&entry_type).expect("Failed to serialize");
            let deserialized: SanctionsEntryType = serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(entry_type, deserialized);
        }
    }
}
