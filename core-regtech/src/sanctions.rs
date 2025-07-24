// =====================================================================================
// File: core-regtech/src/sanctions.rs
// Description: Sanctions screening and watchlist management module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{RegTechError, RegTechResult},
    types::{
        EntityData, MatchType, SanctionsCheck, SanctionsMatch, SanctionsStatus, ScreeningFrequency,
        WatchlistSource,
    },
};

/// Sanctions screening configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsConfig {
    pub watchlist_sources: Vec<WatchlistSource>,
    pub screening_frequency: ScreeningFrequency,
    pub fuzzy_matching_threshold: f64,
    pub auto_block_matches: bool,
    pub manual_review_threshold: f64,
    pub update_frequency_hours: u32,
    pub data_retention_days: u32,
}

/// Sanctions service trait
#[async_trait]
pub trait SanctionsService: Send + Sync {
    /// Screen entity against sanctions lists
    async fn screen_entity(&self, entity_data: &EntityData) -> RegTechResult<SanctionsCheck>;

    /// Batch screen multiple entities
    async fn batch_screen(&self, entities: &[EntityData]) -> RegTechResult<Vec<SanctionsCheck>>;

    /// Update watchlists from external sources
    async fn update_watchlists(&self) -> RegTechResult<WatchlistUpdateResult>;

    /// Get watchlist statistics
    async fn get_watchlist_stats(&self) -> RegTechResult<WatchlistStats>;

    /// Resolve false positive
    async fn resolve_false_positive(&self, check_id: &Uuid, reason: &str) -> RegTechResult<()>;

    /// Add entity to whitelist
    async fn add_to_whitelist(&self, entity_data: &EntityData, reason: &str) -> RegTechResult<()>;
}

/// Watchlist update result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistUpdateResult {
    pub update_id: Uuid,
    pub sources_updated: Vec<WatchlistSource>,
    pub total_entries_added: u32,
    pub total_entries_removed: u32,
    pub total_entries_modified: u32,
    pub update_started_at: chrono::DateTime<chrono::Utc>,
    pub update_completed_at: chrono::DateTime<chrono::Utc>,
    pub errors: Vec<String>,
}

/// Watchlist statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistStats {
    pub total_entries: u32,
    pub entries_by_source: HashMap<WatchlistSource, u32>,
    pub last_update: chrono::DateTime<chrono::Utc>,
    pub update_frequency: ScreeningFrequency,
    pub screening_volume_24h: u32,
    pub match_rate_24h: f64,
}

/// Watchlist entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistEntry {
    pub entry_id: Uuid,
    pub source: WatchlistSource,
    pub name: String,
    pub aliases: Vec<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub place_of_birth: Option<String>,
    pub nationality: Option<String>,
    pub addresses: Vec<crate::types::Address>,
    pub identification_numbers: Vec<crate::types::IdentificationNumber>,
    pub sanctions_type: SanctionsType,
    pub listing_date: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub additional_info: HashMap<String, String>,
}

/// Types of sanctions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SanctionsType {
    Individual,
    Entity,
    Vessel,
    Aircraft,
    PEP, // Politically Exposed Person
    AdverseMedia,
}

/// Sanctions alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsAlert {
    pub alert_id: Uuid,
    pub check_id: Uuid,
    pub entity_id: String,
    pub matches: Vec<SanctionsMatch>,
    pub alert_level: AlertLevel,
    pub requires_manual_review: bool,
    pub auto_blocked: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub resolution: Option<String>,
}

/// Alert levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AlertLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Watchlist manager
pub struct WatchlistManager {
    config: SanctionsConfig,
    watchlist_entries: HashMap<WatchlistSource, Vec<WatchlistEntry>>,
    whitelist: Vec<EntityData>,
    screening_cache: HashMap<String, SanctionsCheck>,
}

/// Sanctions screening service
pub struct SanctionsScreening {
    config: SanctionsConfig,
    watchlist_manager: WatchlistManager,
    fuzzy_matcher: FuzzyMatcher,
}

/// Fuzzy matching service
pub struct FuzzyMatcher {
    threshold: f64,
}

impl FuzzyMatcher {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    pub fn calculate_similarity(&self, text1: &str, text2: &str) -> f64 {
        // Simple Levenshtein distance-based similarity
        let distance = self.levenshtein_distance(text1, text2);
        let max_len = text1.len().max(text2.len()) as f64;

        if max_len == 0.0 {
            1.0
        } else {
            1.0 - (distance as f64 / max_len)
        }
    }

    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1.chars().nth(i - 1) == s2.chars().nth(j - 1) {
                    0
                } else {
                    1
                };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }

    pub fn find_matches(
        &self,
        entity_data: &EntityData,
        watchlist_entries: &[WatchlistEntry],
    ) -> Vec<SanctionsMatch> {
        let mut matches = Vec::new();

        for entry in watchlist_entries {
            // Check name similarity
            let name_similarity = self
                .calculate_similarity(&entity_data.name.to_lowercase(), &entry.name.to_lowercase());

            if name_similarity >= self.threshold {
                matches.push(SanctionsMatch {
                    match_id: Uuid::new_v4(),
                    watchlist_source: entry.source,
                    matched_name: entry.name.clone(),
                    confidence_score: name_similarity,
                    match_type: if name_similarity > 0.95 {
                        MatchType::Exact
                    } else {
                        MatchType::Fuzzy
                    },
                    additional_info: HashMap::new(),
                });
            }

            // Check aliases
            for alias in &entry.aliases {
                let alias_similarity = self
                    .calculate_similarity(&entity_data.name.to_lowercase(), &alias.to_lowercase());

                if alias_similarity >= self.threshold {
                    let mut additional_info = HashMap::new();
                    additional_info.insert("matched_alias".to_string(), alias.clone());

                    matches.push(SanctionsMatch {
                        match_id: Uuid::new_v4(),
                        watchlist_source: entry.source,
                        matched_name: entry.name.clone(),
                        confidence_score: alias_similarity,
                        match_type: MatchType::Alias,
                        additional_info,
                    });
                }
            }
        }

        matches
    }
}

impl WatchlistManager {
    pub fn new(config: SanctionsConfig) -> Self {
        Self {
            config,
            watchlist_entries: HashMap::new(),
            whitelist: Vec::new(),
            screening_cache: HashMap::new(),
        }
    }

    pub async fn initialize(&mut self) -> RegTechResult<()> {
        // Initialize with mock data
        self.load_mock_watchlist_data().await?;
        Ok(())
    }

    async fn load_mock_watchlist_data(&mut self) -> RegTechResult<()> {
        // Mock OFAC entries
        let ofac_entries = vec![
            WatchlistEntry {
                entry_id: Uuid::new_v4(),
                source: WatchlistSource::OFAC,
                name: "John Doe".to_string(),
                aliases: vec!["Johnny Doe".to_string()],
                date_of_birth: Some(chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                place_of_birth: Some("Unknown".to_string()),
                nationality: Some("Unknown".to_string()),
                addresses: Vec::new(),
                identification_numbers: Vec::new(),
                sanctions_type: SanctionsType::Individual,
                listing_date: Utc::now() - chrono::Duration::days(365),
                last_updated: Utc::now(),
                additional_info: HashMap::new(),
            },
            WatchlistEntry {
                entry_id: Uuid::new_v4(),
                source: WatchlistSource::OFAC,
                name: "ACME Corporation".to_string(),
                aliases: vec!["ACME Corp".to_string(), "ACME Ltd".to_string()],
                date_of_birth: None,
                place_of_birth: None,
                nationality: None,
                addresses: Vec::new(),
                identification_numbers: Vec::new(),
                sanctions_type: SanctionsType::Entity,
                listing_date: Utc::now() - chrono::Duration::days(180),
                last_updated: Utc::now(),
                additional_info: HashMap::new(),
            },
        ];

        self.watchlist_entries
            .insert(WatchlistSource::OFAC, ofac_entries);

        // Mock UN entries
        let un_entries = vec![WatchlistEntry {
            entry_id: Uuid::new_v4(),
            source: WatchlistSource::UN,
            name: "Jane Smith".to_string(),
            aliases: vec!["J. Smith".to_string()],
            date_of_birth: Some(chrono::NaiveDate::from_ymd_opt(1975, 5, 15).unwrap()),
            place_of_birth: Some("Unknown".to_string()),
            nationality: Some("Unknown".to_string()),
            addresses: Vec::new(),
            identification_numbers: Vec::new(),
            sanctions_type: SanctionsType::Individual,
            listing_date: Utc::now() - chrono::Duration::days(90),
            last_updated: Utc::now(),
            additional_info: HashMap::new(),
        }];

        self.watchlist_entries
            .insert(WatchlistSource::UN, un_entries);

        Ok(())
    }

    pub fn get_all_entries(&self) -> Vec<&WatchlistEntry> {
        self.watchlist_entries.values().flatten().collect()
    }

    pub fn get_entries_by_source(&self, source: WatchlistSource) -> Option<&Vec<WatchlistEntry>> {
        self.watchlist_entries.get(&source)
    }

    pub fn is_whitelisted(&self, entity_data: &EntityData) -> bool {
        self.whitelist
            .iter()
            .any(|whitelisted| whitelisted.name.to_lowercase() == entity_data.name.to_lowercase())
    }

    pub fn add_to_whitelist(&mut self, entity_data: EntityData) {
        self.whitelist.push(entity_data);
    }

    pub fn get_stats(&self) -> WatchlistStats {
        let total_entries = self
            .watchlist_entries
            .values()
            .map(|entries| entries.len() as u32)
            .sum();
        let entries_by_source = self
            .watchlist_entries
            .iter()
            .map(|(source, entries)| (*source, entries.len() as u32))
            .collect();

        WatchlistStats {
            total_entries,
            entries_by_source,
            last_update: Utc::now(),
            update_frequency: self.config.screening_frequency,
            screening_volume_24h: 100, // Mock value
            match_rate_24h: 0.05,      // Mock value
        }
    }
}

impl SanctionsScreening {
    pub fn new(config: SanctionsConfig) -> Self {
        let watchlist_manager = WatchlistManager::new(config.clone());
        let fuzzy_matcher = FuzzyMatcher::new(config.fuzzy_matching_threshold);

        Self {
            config,
            watchlist_manager,
            fuzzy_matcher,
        }
    }

    pub async fn initialize(&mut self) -> RegTechResult<()> {
        self.watchlist_manager.initialize().await
    }

    fn determine_overall_status(&self, matches: &[SanctionsMatch]) -> SanctionsStatus {
        if matches.is_empty() {
            SanctionsStatus::Clear
        } else {
            let max_confidence = matches
                .iter()
                .map(|m| m.confidence_score)
                .fold(0.0, f64::max);

            if max_confidence >= 0.95 {
                SanctionsStatus::Confirmed
            } else if max_confidence >= self.config.manual_review_threshold {
                SanctionsStatus::UnderReview
            } else {
                SanctionsStatus::PotentialMatch
            }
        }
    }

    fn create_alert_if_needed(&self, check: &SanctionsCheck) -> Option<SanctionsAlert> {
        if check.overall_status == SanctionsStatus::Clear {
            return None;
        }

        let alert_level = match check.confidence_score {
            score if score >= 0.9 => AlertLevel::Critical,
            score if score >= 0.8 => AlertLevel::High,
            score if score >= 0.7 => AlertLevel::Medium,
            _ => AlertLevel::Low,
        };

        let requires_manual_review = check.confidence_score >= self.config.manual_review_threshold;
        let auto_blocked = self.config.auto_block_matches && check.confidence_score >= 0.9;

        Some(SanctionsAlert {
            alert_id: Uuid::new_v4(),
            check_id: check.check_id,
            entity_id: check.entity_id.clone(),
            matches: check.matches.clone(),
            alert_level,
            requires_manual_review,
            auto_blocked,
            created_at: Utc::now(),
            resolved_at: None,
            resolution: None,
        })
    }
}

#[async_trait]
impl SanctionsService for SanctionsScreening {
    async fn screen_entity(&self, entity_data: &EntityData) -> RegTechResult<SanctionsCheck> {
        // Check if entity is whitelisted
        if self.watchlist_manager.is_whitelisted(entity_data) {
            return Ok(SanctionsCheck {
                check_id: Uuid::new_v4(),
                entity_id: entity_data.name.clone(),
                entity_data: entity_data.clone(),
                matches: Vec::new(),
                overall_status: SanctionsStatus::Clear,
                confidence_score: 0.0,
                checked_at: Utc::now(),
                watchlist_version: "v2024.1".to_string(),
            });
        }

        // Perform screening against all watchlists
        let all_entries = self.watchlist_manager.get_all_entries();
        let matches = self.fuzzy_matcher.find_matches(entity_data, &all_entries);

        let overall_status = self.determine_overall_status(&matches);
        let confidence_score = matches
            .iter()
            .map(|m| m.confidence_score)
            .fold(0.0, f64::max);

        let check = SanctionsCheck {
            check_id: Uuid::new_v4(),
            entity_id: entity_data.name.clone(),
            entity_data: entity_data.clone(),
            matches,
            overall_status,
            confidence_score,
            checked_at: Utc::now(),
            watchlist_version: "v2024.1".to_string(),
        };

        // Create alert if needed
        if let Some(_alert) = self.create_alert_if_needed(&check) {
            // In a real implementation, this would be stored and notifications sent
        }

        Ok(check)
    }

    async fn batch_screen(&self, entities: &[EntityData]) -> RegTechResult<Vec<SanctionsCheck>> {
        let mut results = Vec::new();

        for entity in entities {
            let check = self.screen_entity(entity).await?;
            results.push(check);
        }

        Ok(results)
    }

    async fn update_watchlists(&self) -> RegTechResult<WatchlistUpdateResult> {
        let update_started_at = Utc::now();

        // Mock watchlist update
        let result = WatchlistUpdateResult {
            update_id: Uuid::new_v4(),
            sources_updated: self.config.watchlist_sources.clone(),
            total_entries_added: 50,
            total_entries_removed: 5,
            total_entries_modified: 20,
            update_started_at,
            update_completed_at: Utc::now(),
            errors: Vec::new(),
        };

        Ok(result)
    }

    async fn get_watchlist_stats(&self) -> RegTechResult<WatchlistStats> {
        Ok(self.watchlist_manager.get_stats())
    }

    async fn resolve_false_positive(&self, check_id: &Uuid, reason: &str) -> RegTechResult<()> {
        // Mock false positive resolution
        Ok(())
    }

    async fn add_to_whitelist(&self, entity_data: &EntityData, reason: &str) -> RegTechResult<()> {
        // Mock whitelist addition
        Ok(())
    }
}

impl Default for SanctionsConfig {
    fn default() -> Self {
        Self {
            watchlist_sources: vec![
                WatchlistSource::OFAC,
                WatchlistSource::UN,
                WatchlistSource::EU,
            ],
            screening_frequency: ScreeningFrequency::RealTime,
            fuzzy_matching_threshold: 0.8,
            auto_block_matches: false,
            manual_review_threshold: 0.7,
            update_frequency_hours: 24,
            data_retention_days: 2555, // 7 years
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_matcher() {
        let matcher = FuzzyMatcher::new(0.8);

        // Test exact match
        let similarity = matcher.calculate_similarity("John Doe", "John Doe");
        assert_eq!(similarity, 1.0);

        // Test similar names
        let similarity = matcher.calculate_similarity("John Doe", "Jon Doe");
        assert!(similarity > 0.8);

        // Test different names
        let similarity = matcher.calculate_similarity("John Doe", "Jane Smith");
        assert!(similarity < 0.5);
    }

    #[tokio::test]
    async fn test_sanctions_screening() {
        let config = SanctionsConfig::default();
        let mut service = SanctionsScreening::new(config);
        service.initialize().await.unwrap();

        let entity_data = EntityData {
            name: "John Doe".to_string(),
            aliases: vec![],
            date_of_birth: None,
            place_of_birth: None,
            nationality: None,
            addresses: vec![],
            identification_numbers: vec![],
        };

        let result = service.screen_entity(&entity_data).await;
        assert!(result.is_ok());

        let check = result.unwrap();
        assert_eq!(check.entity_id, "John Doe");
        assert!(!check.matches.is_empty()); // Should match mock data
        assert_ne!(check.overall_status, SanctionsStatus::Clear);
    }

    #[tokio::test]
    async fn test_batch_screening() {
        let config = SanctionsConfig::default();
        let mut service = SanctionsScreening::new(config);
        service.initialize().await.unwrap();

        let entities = vec![
            EntityData {
                name: "John Doe".to_string(),
                aliases: vec![],
                date_of_birth: None,
                place_of_birth: None,
                nationality: None,
                addresses: vec![],
                identification_numbers: vec![],
            },
            EntityData {
                name: "Clean Entity".to_string(),
                aliases: vec![],
                date_of_birth: None,
                place_of_birth: None,
                nationality: None,
                addresses: vec![],
                identification_numbers: vec![],
            },
        ];

        let result = service.batch_screen(&entities).await;
        assert!(result.is_ok());

        let checks = result.unwrap();
        assert_eq!(checks.len(), 2);

        // First entity should have matches
        assert!(!checks[0].matches.is_empty());

        // Second entity should be clear
        assert!(checks[1].matches.is_empty());
        assert_eq!(checks[1].overall_status, SanctionsStatus::Clear);
    }

    #[tokio::test]
    async fn test_watchlist_stats() {
        let config = SanctionsConfig::default();
        let mut service = SanctionsScreening::new(config);
        service.initialize().await.unwrap();

        let stats = service.get_watchlist_stats().await.unwrap();
        assert!(stats.total_entries > 0);
        assert!(stats.entries_by_source.contains_key(&WatchlistSource::OFAC));
        assert!(stats.entries_by_source.contains_key(&WatchlistSource::UN));
    }
}
