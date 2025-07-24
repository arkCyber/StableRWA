// =====================================================================================
// IPFS Metadata Management
//
// Advanced metadata operations for content stored in IPFS
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{ContentMetadata, IpfsError, IpfsHash, IpfsResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, instrument};
use uuid::Uuid;

/// Metadata schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataSchema {
    pub name: String,
    pub version: String,
    pub fields: HashMap<String, FieldDefinition>,
    pub required_fields: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Field definition for metadata schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub field_type: FieldType,
    pub description: String,
    pub validation_rules: Vec<ValidationRule>,
    pub default_value: Option<serde_json::Value>,
}

/// Field types supported in metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    DateTime,
    Url,
    Hash,
}

/// Validation rules for metadata fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    Range(f64, f64),
    OneOf(Vec<serde_json::Value>),
    Required,
}

/// Metadata query builder
#[derive(Debug, Clone, Default)]
pub struct MetadataQuery {
    pub tags: Vec<String>,
    pub mime_types: Vec<String>,
    pub size_range: Option<(u64, u64)>,
    pub date_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    pub custom_filters: HashMap<String, serde_json::Value>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl MetadataQuery {
    /// Create new query builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add tag filter
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Add MIME type filter
    pub fn with_mime_type(mut self, mime_type: String) -> Self {
        self.mime_types.push(mime_type);
        self
    }

    /// Add size range filter
    pub fn with_size_range(mut self, min: u64, max: u64) -> Self {
        self.size_range = Some((min, max));
        self
    }

    /// Add date range filter
    pub fn with_date_range(
        mut self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        self.date_range = Some((start, end));
        self
    }

    /// Add custom field filter
    pub fn with_custom_filter(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom_filters.insert(key, value);
        self
    }

    /// Set pagination
    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

/// Metadata manager trait
#[async_trait(?Send)]
pub trait MetadataManagerTrait {
    /// Create metadata schema
    async fn create_schema(&self, schema: MetadataSchema) -> IpfsResult<()>;

    /// Get metadata schema
    async fn get_schema(&self, name: &str) -> IpfsResult<MetadataSchema>;

    /// Validate metadata against schema
    async fn validate_metadata(
        &self,
        metadata: &ContentMetadata,
        schema_name: &str,
    ) -> IpfsResult<()>;

    /// Query metadata with complex filters
    async fn query_metadata(&self, query: MetadataQuery) -> IpfsResult<Vec<ContentMetadata>>;

    /// Bulk update metadata
    async fn bulk_update_metadata(
        &self,
        updates: HashMap<IpfsHash, ContentMetadata>,
    ) -> IpfsResult<()>;

    /// Export metadata to JSON
    async fn export_metadata(&self, hashes: Option<Vec<IpfsHash>>) -> IpfsResult<String>;

    /// Import metadata from JSON
    async fn import_metadata(&self, json_data: &str) -> IpfsResult<Vec<IpfsHash>>;

    /// Get metadata statistics
    async fn get_metadata_stats(&self) -> IpfsResult<MetadataStats>;
}

/// Metadata statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataStats {
    pub total_items: u64,
    pub schemas_count: u64,
    pub tags_distribution: HashMap<String, u64>,
    pub mime_types_distribution: HashMap<String, u64>,
    pub size_distribution: SizeDistribution,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Size distribution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeDistribution {
    pub small_files: u64,  // < 1MB
    pub medium_files: u64, // 1MB - 10MB
    pub large_files: u64,  // > 10MB
    pub total_size: u64,
    pub average_size: f64,
}

/// In-memory metadata manager implementation
pub struct InMemoryMetadataManager {
    metadata_store: std::sync::Arc<tokio::sync::RwLock<HashMap<String, ContentMetadata>>>,
    schema_store: std::sync::Arc<tokio::sync::RwLock<HashMap<String, MetadataSchema>>>,
}

impl InMemoryMetadataManager {
    /// Create new in-memory metadata manager
    pub fn new() -> Self {
        Self {
            metadata_store: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            schema_store: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Add metadata to store
    pub async fn add_metadata(&self, metadata: ContentMetadata) {
        self.metadata_store
            .write()
            .await
            .insert(metadata.hash.as_str().to_string(), metadata);
    }

    /// Remove metadata from store
    pub async fn remove_metadata(&self, hash: &IpfsHash) {
        self.metadata_store.write().await.remove(hash.as_str());
    }

    /// Get all metadata
    pub async fn get_all_metadata(&self) -> Vec<ContentMetadata> {
        self.metadata_store.read().await.values().cloned().collect()
    }

    /// Validate field value against definition
    fn validate_field_value(
        &self,
        value: &serde_json::Value,
        definition: &FieldDefinition,
    ) -> IpfsResult<()> {
        for rule in &definition.validation_rules {
            match rule {
                ValidationRule::Required => {
                    if value.is_null() {
                        return Err(IpfsError::ValidationError(
                            "Required field is missing".to_string(),
                        ));
                    }
                }
                ValidationRule::MinLength(min) => {
                    if let Some(s) = value.as_str() {
                        if s.len() < *min {
                            return Err(IpfsError::ValidationError(format!(
                                "String length {} is less than minimum {}",
                                s.len(),
                                min
                            )));
                        }
                    }
                }
                ValidationRule::MaxLength(max) => {
                    if let Some(s) = value.as_str() {
                        if s.len() > *max {
                            return Err(IpfsError::ValidationError(format!(
                                "String length {} exceeds maximum {}",
                                s.len(),
                                max
                            )));
                        }
                    }
                }
                ValidationRule::Pattern(pattern) => {
                    if let Some(s) = value.as_str() {
                        let regex = regex::Regex::new(pattern).map_err(|e| {
                            IpfsError::ValidationError(format!("Invalid regex pattern: {}", e))
                        })?;
                        if !regex.is_match(s) {
                            return Err(IpfsError::ValidationError(format!(
                                "String '{}' does not match pattern '{}'",
                                s, pattern
                            )));
                        }
                    }
                }
                ValidationRule::Range(min, max) => {
                    if let Some(n) = value.as_f64() {
                        if n < *min || n > *max {
                            return Err(IpfsError::ValidationError(format!(
                                "Number {} is outside range [{}, {}]",
                                n, min, max
                            )));
                        }
                    }
                }
                ValidationRule::OneOf(allowed_values) => {
                    if !allowed_values.contains(value) {
                        return Err(IpfsError::ValidationError(format!(
                            "Value {:?} is not in allowed list",
                            value
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    /// Check if metadata matches query
    fn matches_query(&self, metadata: &ContentMetadata, query: &MetadataQuery) -> bool {
        // Check tags
        if !query.tags.is_empty() {
            let has_matching_tag = query.tags.iter().any(|tag| metadata.tags.contains(tag));
            if !has_matching_tag {
                return false;
            }
        }

        // Check MIME types
        if !query.mime_types.is_empty() && !query.mime_types.contains(&metadata.mime_type) {
            return false;
        }

        // Check size range
        if let Some((min, max)) = query.size_range {
            if metadata.size < min || metadata.size > max {
                return false;
            }
        }

        // Check date range
        if let Some((start, end)) = query.date_range {
            if metadata.created_at < start || metadata.created_at > end {
                return false;
            }
        }

        // Check custom filters
        for (key, expected_value) in &query.custom_filters {
            if let Some(actual_value) = metadata.custom_fields.get(key) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

#[async_trait(?Send)]
impl MetadataManagerTrait for InMemoryMetadataManager {
    #[instrument(skip(self, schema))]
    async fn create_schema(&self, schema: MetadataSchema) -> IpfsResult<()> {
        debug!("Creating metadata schema: {}", schema.name);

        self.schema_store
            .write()
            .await
            .insert(schema.name.clone(), schema);

        info!("Successfully created metadata schema");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_schema(&self, name: &str) -> IpfsResult<MetadataSchema> {
        debug!("Getting metadata schema: {}", name);

        self.schema_store
            .read()
            .await
            .get(name)
            .cloned()
            .ok_or_else(|| IpfsError::MetadataError(format!("Schema not found: {}", name)))
    }

    #[instrument(skip(self, metadata))]
    async fn validate_metadata(
        &self,
        metadata: &ContentMetadata,
        schema_name: &str,
    ) -> IpfsResult<()> {
        debug!("Validating metadata against schema: {}", schema_name);

        let schema = self.get_schema(schema_name).await?;

        // Check required fields
        for required_field in &schema.required_fields {
            if !metadata.custom_fields.contains_key(required_field) {
                return Err(IpfsError::ValidationError(format!(
                    "Required field '{}' is missing",
                    required_field
                )));
            }
        }

        // Validate field values
        for (field_name, field_value) in &metadata.custom_fields {
            if let Some(field_def) = schema.fields.get(field_name) {
                self.validate_field_value(field_value, field_def)?;
            }
        }

        info!("Metadata validation successful");
        Ok(())
    }

    #[instrument(skip(self, query))]
    async fn query_metadata(&self, query: MetadataQuery) -> IpfsResult<Vec<ContentMetadata>> {
        debug!("Querying metadata with filters");

        let metadata_store = self.metadata_store.read().await;
        let mut results: Vec<ContentMetadata> = metadata_store
            .values()
            .filter(|metadata| self.matches_query(metadata, &query))
            .cloned()
            .collect();

        // Apply pagination
        if let Some(offset) = query.offset {
            if offset < results.len() {
                results = results.into_iter().skip(offset).collect();
            } else {
                results.clear();
            }
        }

        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        info!("Query returned {} results", results.len());
        Ok(results)
    }

    #[instrument(skip(self, updates))]
    async fn bulk_update_metadata(
        &self,
        updates: HashMap<IpfsHash, ContentMetadata>,
    ) -> IpfsResult<()> {
        debug!("Bulk updating {} metadata items", updates.len());

        let mut metadata_store = self.metadata_store.write().await;
        for (hash, metadata) in updates {
            metadata_store.insert(hash.as_str().to_string(), metadata);
        }

        info!(
            "Successfully updated metadata for {} items",
            metadata_store.len()
        );
        Ok(())
    }

    #[instrument(skip(self))]
    async fn export_metadata(&self, hashes: Option<Vec<IpfsHash>>) -> IpfsResult<String> {
        debug!("Exporting metadata to JSON");

        let metadata_store = self.metadata_store.read().await;
        let metadata_to_export: Vec<ContentMetadata> = match hashes {
            Some(hash_list) => hash_list
                .iter()
                .filter_map(|hash| metadata_store.get(hash.as_str()).cloned())
                .collect(),
            None => metadata_store.values().cloned().collect(),
        };

        let json = serde_json::to_string_pretty(&metadata_to_export).map_err(|e| {
            IpfsError::SerializationError(format!("Failed to serialize metadata: {}", e))
        })?;

        info!(
            "Exported {} metadata items to JSON",
            metadata_to_export.len()
        );
        Ok(json)
    }

    #[instrument(skip(self, json_data))]
    async fn import_metadata(&self, json_data: &str) -> IpfsResult<Vec<IpfsHash>> {
        debug!("Importing metadata from JSON");

        let metadata_list: Vec<ContentMetadata> = serde_json::from_str(json_data).map_err(|e| {
            IpfsError::SerializationError(format!("Failed to deserialize metadata: {}", e))
        })?;

        let mut imported_hashes = Vec::new();
        let mut metadata_store = self.metadata_store.write().await;

        for metadata in metadata_list {
            let hash = metadata.hash.clone();
            metadata_store.insert(hash.as_str().to_string(), metadata);
            imported_hashes.push(hash);
        }

        info!(
            "Imported {} metadata items from JSON",
            imported_hashes.len()
        );
        Ok(imported_hashes)
    }

    #[instrument(skip(self))]
    async fn get_metadata_stats(&self) -> IpfsResult<MetadataStats> {
        debug!("Calculating metadata statistics");

        let metadata_store = self.metadata_store.read().await;
        let schema_store = self.schema_store.read().await;

        let total_items = metadata_store.len() as u64;
        let schemas_count = schema_store.len() as u64;

        // Calculate tag distribution
        let mut tags_distribution = HashMap::new();
        for metadata in metadata_store.values() {
            for tag in &metadata.tags {
                *tags_distribution.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        // Calculate MIME type distribution
        let mut mime_types_distribution = HashMap::new();
        for metadata in metadata_store.values() {
            *mime_types_distribution
                .entry(metadata.mime_type.clone())
                .or_insert(0) += 1;
        }

        // Calculate size distribution
        let mut small_files = 0;
        let mut medium_files = 0;
        let mut large_files = 0;
        let mut total_size = 0;

        for metadata in metadata_store.values() {
            total_size += metadata.size;
            match metadata.size {
                0..=1_048_576 => small_files += 1,           // < 1MB
                1_048_577..=10_485_760 => medium_files += 1, // 1MB - 10MB
                _ => large_files += 1,                       // > 10MB
            }
        }

        let average_size = if total_items > 0 {
            total_size as f64 / total_items as f64
        } else {
            0.0
        };

        let size_distribution = SizeDistribution {
            small_files,
            medium_files,
            large_files,
            total_size,
            average_size,
        };

        let stats = MetadataStats {
            total_items,
            schemas_count,
            tags_distribution,
            mime_types_distribution,
            size_distribution,
            last_updated: chrono::Utc::now(),
        };

        info!(
            "Generated metadata statistics: {} items, {} schemas",
            total_items, schemas_count
        );
        Ok(stats)
    }
}

impl Default for InMemoryMetadataManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata() -> ContentMetadata {
        let hash =
            IpfsHash::new("QmTestMetadata123456789012345678901234567890".to_string()).unwrap();
        let mut metadata = ContentMetadata::new(
            hash,
            "test_file.txt".to_string(),
            1024,
            "text/plain".to_string(),
        );
        metadata.add_tag("test".to_string());
        metadata.add_tag("metadata".to_string());
        metadata.add_custom_field(
            "author".to_string(),
            serde_json::Value::String("test_user".to_string()),
        );
        metadata.add_custom_field(
            "version".to_string(),
            serde_json::Value::Number(serde_json::Number::from(1)),
        );
        metadata
    }

    fn create_test_schema() -> MetadataSchema {
        let mut fields = HashMap::new();

        fields.insert(
            "author".to_string(),
            FieldDefinition {
                field_type: FieldType::String,
                description: "Author of the content".to_string(),
                validation_rules: vec![
                    ValidationRule::Required,
                    ValidationRule::MinLength(1),
                    ValidationRule::MaxLength(100),
                ],
                default_value: None,
            },
        );

        fields.insert(
            "version".to_string(),
            FieldDefinition {
                field_type: FieldType::Number,
                description: "Version number".to_string(),
                validation_rules: vec![ValidationRule::Range(1.0, 100.0)],
                default_value: Some(serde_json::Value::Number(serde_json::Number::from(1))),
            },
        );

        MetadataSchema {
            name: "test_schema".to_string(),
            version: "1.0".to_string(),
            fields,
            required_fields: vec!["author".to_string()],
            created_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_metadata_query_builder() {
        let query = MetadataQuery::new()
            .with_tag("test".to_string())
            .with_mime_type("text/plain".to_string())
            .with_size_range(100, 2000)
            .with_custom_filter(
                "author".to_string(),
                serde_json::Value::String("test_user".to_string()),
            )
            .with_pagination(10, 0);

        assert_eq!(query.tags.len(), 1);
        assert_eq!(query.mime_types.len(), 1);
        assert_eq!(query.size_range, Some((100, 2000)));
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(0));
    }

    #[tokio::test]
    async fn test_schema_operations() {
        let manager = InMemoryMetadataManager::new();
        let schema = create_test_schema();

        // Create schema
        manager.create_schema(schema.clone()).await.unwrap();

        // Get schema
        let retrieved_schema = manager.get_schema("test_schema").await.unwrap();
        assert_eq!(retrieved_schema.name, schema.name);
        assert_eq!(retrieved_schema.version, schema.version);
        assert_eq!(retrieved_schema.fields.len(), schema.fields.len());

        // Try to get non-existent schema
        assert!(manager.get_schema("non_existent").await.is_err());
    }

    #[tokio::test]
    async fn test_metadata_validation() {
        let manager = InMemoryMetadataManager::new();
        let schema = create_test_schema();
        manager.create_schema(schema).await.unwrap();

        let metadata = create_test_metadata();

        // Valid metadata should pass
        assert!(manager
            .validate_metadata(&metadata, "test_schema")
            .await
            .is_ok());

        // Create invalid metadata (missing required field)
        let hash =
            IpfsHash::new("QmInvalidMetadata123456789012345678901234567".to_string()).unwrap();
        let invalid_metadata = ContentMetadata::new(
            hash,
            "invalid.txt".to_string(),
            1024,
            "text/plain".to_string(),
        );

        // Should fail validation
        assert!(manager
            .validate_metadata(&invalid_metadata, "test_schema")
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_metadata_query() {
        let manager = InMemoryMetadataManager::new();

        // Add test metadata
        let metadata1 = create_test_metadata();
        let mut metadata2 = create_test_metadata();
        metadata2.hash =
            IpfsHash::new("QmTestMetadata234567890123456789012345678901".to_string()).unwrap();
        metadata2.name = "another_file.txt".to_string();
        metadata2.mime_type = "application/json".to_string();
        metadata2.tags = vec!["json".to_string(), "data".to_string()];

        manager.add_metadata(metadata1.clone()).await;
        manager.add_metadata(metadata2.clone()).await;

        // Query by tag
        let query = MetadataQuery::new().with_tag("test".to_string());
        let results = manager.query_metadata(query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test_file.txt");

        // Query by MIME type
        let query = MetadataQuery::new().with_mime_type("application/json".to_string());
        let results = manager.query_metadata(query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "another_file.txt");

        // Query with pagination
        let query = MetadataQuery::new().with_pagination(1, 0);
        let results = manager.query_metadata(query).await.unwrap();
        assert_eq!(results.len(), 1);

        let query = MetadataQuery::new().with_pagination(1, 1);
        let results = manager.query_metadata(query).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_bulk_operations() {
        let manager = InMemoryMetadataManager::new();

        // Create multiple metadata items
        let mut updates = HashMap::new();
        for i in 0..3 {
            let hash = IpfsHash::new(format!("QmBulkTest{:040}", i)).unwrap();
            let metadata = ContentMetadata::new(
                hash.clone(),
                format!("file_{}.txt", i),
                1024 * (i as u64 + 1),
                "text/plain".to_string(),
            );
            updates.insert(hash, metadata);
        }

        // Bulk update
        manager.bulk_update_metadata(updates).await.unwrap();

        // Verify all items were added
        let all_metadata = manager.get_all_metadata().await;
        assert_eq!(all_metadata.len(), 3);
    }

    #[tokio::test]
    async fn test_export_import_metadata() {
        let manager = InMemoryMetadataManager::new();

        // Add test metadata
        let metadata = create_test_metadata();
        manager.add_metadata(metadata.clone()).await;

        // Export metadata
        let json_export = manager.export_metadata(None).await.unwrap();
        assert!(!json_export.is_empty());

        // Clear metadata store
        manager.remove_metadata(&metadata.hash).await;
        assert_eq!(manager.get_all_metadata().await.len(), 0);

        // Import metadata
        let imported_hashes = manager.import_metadata(&json_export).await.unwrap();
        assert_eq!(imported_hashes.len(), 1);
        assert_eq!(imported_hashes[0], metadata.hash);

        // Verify imported metadata
        let all_metadata = manager.get_all_metadata().await;
        assert_eq!(all_metadata.len(), 1);
        assert_eq!(all_metadata[0].name, metadata.name);
    }

    #[tokio::test]
    async fn test_metadata_statistics() {
        let manager = InMemoryMetadataManager::new();

        // Add test data with different characteristics
        let mut metadata1 = create_test_metadata();
        metadata1.size = 500_000; // Small file
        metadata1.mime_type = "image/jpeg".to_string();

        let mut metadata2 = create_test_metadata();
        metadata2.hash =
            IpfsHash::new("QmStats234567890123456789012345678901234567".to_string()).unwrap();
        metadata2.size = 5_000_000; // Medium file
        metadata2.mime_type = "image/jpeg".to_string();
        metadata2.tags = vec!["image".to_string(), "photo".to_string()];

        let mut metadata3 = create_test_metadata();
        metadata3.hash =
            IpfsHash::new("QmStats345678901234567890123456789012345678".to_string()).unwrap();
        metadata3.size = 50_000_000; // Large file
        metadata3.mime_type = "video/mp4".to_string();
        metadata3.tags = vec!["video".to_string(), "media".to_string()];

        manager.add_metadata(metadata1).await;
        manager.add_metadata(metadata2).await;
        manager.add_metadata(metadata3).await;

        // Create a test schema
        let schema = create_test_schema();
        manager.create_schema(schema).await.unwrap();

        // Get statistics
        let stats = manager.get_metadata_stats().await.unwrap();

        assert_eq!(stats.total_items, 3);
        assert_eq!(stats.schemas_count, 1);
        assert_eq!(stats.size_distribution.small_files, 1);
        assert_eq!(stats.size_distribution.medium_files, 1);
        assert_eq!(stats.size_distribution.large_files, 1);
        assert_eq!(stats.size_distribution.total_size, 55_500_000);

        // Check tag distribution
        assert!(stats.tags_distribution.contains_key("test"));
        assert!(stats.tags_distribution.contains_key("image"));
        assert!(stats.tags_distribution.contains_key("video"));

        // Check MIME type distribution
        assert_eq!(stats.mime_types_distribution.get("image/jpeg"), Some(&2));
        assert_eq!(stats.mime_types_distribution.get("video/mp4"), Some(&1));
    }

    #[test]
    fn test_field_types_and_validation_rules() {
        // Test field type serialization
        let field_type = FieldType::String;
        let serialized = serde_json::to_string(&field_type).unwrap();
        assert!(serialized.contains("String"));

        // Test validation rule serialization
        let rule = ValidationRule::MinLength(5);
        let serialized = serde_json::to_string(&rule).unwrap();
        assert!(serialized.contains("MinLength"));
    }
}
