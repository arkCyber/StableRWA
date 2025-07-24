// =====================================================================================
// File: core-regtech/src/document_analysis.rs
// Description: Document analysis and classification module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{RegTechError, RegTechResult},
    types::DocumentType,
};

/// Document analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentConfig {
    pub supported_formats: Vec<String>,
    pub max_file_size_mb: u32,
    pub ocr_enabled: bool,
    pub ai_classification: bool,
}

/// Document analyzer trait
#[async_trait]
pub trait DocumentAnalyzer: Send + Sync {
    /// Analyze document
    async fn analyze_document(&self, document: &DocumentInput) -> RegTechResult<DocumentAnalysis>;

    /// Extract text from document
    async fn extract_text(&self, document: &DocumentInput) -> RegTechResult<String>;

    /// Classify document type
    async fn classify_document(
        &self,
        document: &DocumentInput,
    ) -> RegTechResult<DocumentClassification>;

    /// Validate document
    async fn validate_document(
        &self,
        document: &DocumentInput,
    ) -> RegTechResult<DocumentValidation>;
}

/// Document input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInput {
    pub document_id: Uuid,
    pub file_name: String,
    pub file_type: String,
    pub file_size: u64,
    pub content: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

/// Document analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentAnalysis {
    pub analysis_id: Uuid,
    pub document_id: Uuid,
    pub classification: DocumentClassification,
    pub text_extraction: TextExtraction,
    pub validation: DocumentValidation,
    pub analyzed_at: chrono::DateTime<chrono::Utc>,
}

/// Document classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentClassification {
    pub document_type: DocumentType,
    pub confidence_score: f64,
    pub alternative_types: Vec<(DocumentType, f64)>,
    pub classification_method: ClassificationMethod,
}

/// Classification methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClassificationMethod {
    RuleBased,
    MachineLearning,
    Hybrid,
}

/// Text extraction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextExtraction {
    pub extracted_text: String,
    pub confidence_score: f64,
    pub language: String,
    pub extraction_method: ExtractionMethod,
    pub structured_data: HashMap<String, String>,
}

/// Extraction methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExtractionMethod {
    OCR,
    NativeText,
    Hybrid,
}

/// Document validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentValidation {
    pub is_valid: bool,
    pub validation_score: f64,
    pub issues: Vec<ValidationIssue>,
    pub security_features: Vec<SecurityFeature>,
}

/// Validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub issue_type: IssueType,
    pub severity: IssueSeverity,
    pub description: String,
    pub location: Option<String>,
}

/// Issue types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueType {
    FormatError,
    QualityIssue,
    SecurityConcern,
    DataInconsistency,
    MissingInformation,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Security feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFeature {
    pub feature_type: SecurityFeatureType,
    pub present: bool,
    pub confidence: f64,
    pub description: String,
}

/// Security feature types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityFeatureType {
    Watermark,
    Hologram,
    MicroText,
    SecurityThread,
    SpecialPaper,
    DigitalSignature,
}

/// Document analyzer implementation
pub struct DocumentAnalyzerImpl {
    config: DocumentConfig,
}

impl DocumentAnalyzerImpl {
    pub fn new(config: DocumentConfig) -> Self {
        Self { config }
    }

    fn classify_by_filename(&self, filename: &str) -> DocumentType {
        let filename_lower = filename.to_lowercase();

        if filename_lower.contains("passport") {
            DocumentType::Passport
        } else if filename_lower.contains("license") || filename_lower.contains("driver") {
            DocumentType::DriverLicense
        } else if filename_lower.contains("id") || filename_lower.contains("national") {
            DocumentType::NationalID
        } else if filename_lower.contains("utility") || filename_lower.contains("bill") {
            DocumentType::UtilityBill
        } else if filename_lower.contains("bank") || filename_lower.contains("statement") {
            DocumentType::BankStatement
        } else {
            DocumentType::NationalID // Default
        }
    }

    fn extract_text_mock(&self, document: &DocumentInput) -> String {
        // Mock text extraction - in reality, this would use OCR or native text extraction
        format!("Extracted text from document: {}", document.file_name)
    }

    fn validate_document_mock(&self, document: &DocumentInput) -> DocumentValidation {
        let mut issues = Vec::new();

        // Check file size
        if document.file_size > (self.config.max_file_size_mb as u64 * 1024 * 1024) {
            issues.push(ValidationIssue {
                issue_type: IssueType::FormatError,
                severity: IssueSeverity::High,
                description: "File size exceeds maximum allowed".to_string(),
                location: None,
            });
        }

        // Check file format
        if !self.config.supported_formats.contains(&document.file_type) {
            issues.push(ValidationIssue {
                issue_type: IssueType::FormatError,
                severity: IssueSeverity::Critical,
                description: "Unsupported file format".to_string(),
                location: None,
            });
        }

        let is_valid = issues
            .iter()
            .all(|issue| issue.severity < IssueSeverity::Critical);
        let validation_score = if is_valid { 0.9 } else { 0.3 };

        DocumentValidation {
            is_valid,
            validation_score,
            issues,
            security_features: vec![SecurityFeature {
                feature_type: SecurityFeatureType::Watermark,
                present: true,
                confidence: 0.8,
                description: "Watermark detected".to_string(),
            }],
        }
    }
}

#[async_trait]
impl DocumentAnalyzer for DocumentAnalyzerImpl {
    async fn analyze_document(&self, document: &DocumentInput) -> RegTechResult<DocumentAnalysis> {
        let classification = self.classify_document(document).await?;
        let text_extraction = TextExtraction {
            extracted_text: self.extract_text_mock(document),
            confidence_score: 0.95,
            language: "en".to_string(),
            extraction_method: if self.config.ocr_enabled {
                ExtractionMethod::OCR
            } else {
                ExtractionMethod::NativeText
            },
            structured_data: HashMap::new(),
        };
        let validation = self.validate_document_mock(document);

        Ok(DocumentAnalysis {
            analysis_id: Uuid::new_v4(),
            document_id: document.document_id,
            classification,
            text_extraction,
            validation,
            analyzed_at: chrono::Utc::now(),
        })
    }

    async fn extract_text(&self, document: &DocumentInput) -> RegTechResult<String> {
        Ok(self.extract_text_mock(document))
    }

    async fn classify_document(
        &self,
        document: &DocumentInput,
    ) -> RegTechResult<DocumentClassification> {
        let document_type = self.classify_by_filename(&document.file_name);

        Ok(DocumentClassification {
            document_type,
            confidence_score: 0.85,
            alternative_types: vec![(DocumentType::NationalID, 0.15)],
            classification_method: if self.config.ai_classification {
                ClassificationMethod::MachineLearning
            } else {
                ClassificationMethod::RuleBased
            },
        })
    }

    async fn validate_document(
        &self,
        document: &DocumentInput,
    ) -> RegTechResult<DocumentValidation> {
        Ok(self.validate_document_mock(document))
    }
}

impl Default for DocumentConfig {
    fn default() -> Self {
        Self {
            supported_formats: vec![
                "pdf".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "tiff".to_string(),
            ],
            max_file_size_mb: 10,
            ocr_enabled: true,
            ai_classification: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_document_analysis() {
        let config = DocumentConfig::default();
        let analyzer = DocumentAnalyzerImpl::new(config);

        let document = DocumentInput {
            document_id: Uuid::new_v4(),
            file_name: "passport.pdf".to_string(),
            file_type: "pdf".to_string(),
            file_size: 1024 * 1024, // 1MB
            content: vec![0u8; 1024],
            metadata: HashMap::new(),
        };

        let result = analyzer.analyze_document(&document).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.document_id, document.document_id);
        assert_eq!(
            analysis.classification.document_type,
            DocumentType::Passport
        );
        assert!(analysis.validation.is_valid);
    }

    #[tokio::test]
    async fn test_document_classification() {
        let config = DocumentConfig::default();
        let analyzer = DocumentAnalyzerImpl::new(config);

        let document = DocumentInput {
            document_id: Uuid::new_v4(),
            file_name: "driver_license.jpg".to_string(),
            file_type: "jpg".to_string(),
            file_size: 512 * 1024, // 512KB
            content: vec![0u8; 512],
            metadata: HashMap::new(),
        };

        let result = analyzer.classify_document(&document).await;
        assert!(result.is_ok());

        let classification = result.unwrap();
        assert_eq!(classification.document_type, DocumentType::DriverLicense);
        assert!(classification.confidence_score > 0.8);
    }

    #[tokio::test]
    async fn test_text_extraction() {
        let config = DocumentConfig::default();
        let analyzer = DocumentAnalyzerImpl::new(config);

        let document = DocumentInput {
            document_id: Uuid::new_v4(),
            file_name: "test_document.pdf".to_string(),
            file_type: "pdf".to_string(),
            file_size: 1024,
            content: vec![0u8; 1024],
            metadata: HashMap::new(),
        };

        let result = analyzer.extract_text(&document).await;
        assert!(result.is_ok());

        let text = result.unwrap();
        assert!(!text.is_empty());
        assert!(text.contains("test_document.pdf"));
    }

    #[tokio::test]
    async fn test_document_validation() {
        let config = DocumentConfig::default();
        let analyzer = DocumentAnalyzerImpl::new(config);

        // Test valid document
        let valid_document = DocumentInput {
            document_id: Uuid::new_v4(),
            file_name: "valid.pdf".to_string(),
            file_type: "pdf".to_string(),
            file_size: 1024 * 1024, // 1MB
            content: vec![0u8; 1024],
            metadata: HashMap::new(),
        };

        let result = analyzer.validate_document(&valid_document).await;
        assert!(result.is_ok());

        let validation = result.unwrap();
        assert!(validation.is_valid);
        assert!(validation.validation_score > 0.8);

        // Test invalid document (unsupported format)
        let invalid_document = DocumentInput {
            document_id: Uuid::new_v4(),
            file_name: "invalid.txt".to_string(),
            file_type: "txt".to_string(),
            file_size: 1024,
            content: vec![0u8; 1024],
            metadata: HashMap::new(),
        };

        let result = analyzer.validate_document(&invalid_document).await;
        assert!(result.is_ok());

        let validation = result.unwrap();
        assert!(!validation.is_valid);
        assert!(!validation.issues.is_empty());
    }
}
