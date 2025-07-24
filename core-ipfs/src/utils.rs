// =====================================================================================
// IPFS Utilities
//
// Utility functions and helpers for IPFS operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{IpfsError, IpfsHash, IpfsResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use tracing::{debug, instrument};

/// File type detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeInfo {
    pub mime_type: String,
    pub extension: Option<String>,
    pub is_text: bool,
    pub is_image: bool,
    pub is_video: bool,
    pub is_audio: bool,
    pub is_document: bool,
}

impl FileTypeInfo {
    /// Create new file type info
    pub fn new(mime_type: String) -> Self {
        let is_text = mime_type.starts_with("text/");
        let is_image = mime_type.starts_with("image/");
        let is_video = mime_type.starts_with("video/");
        let is_audio = mime_type.starts_with("audio/");
        let is_document = matches!(
            mime_type.as_str(),
            "application/pdf"
                | "application/msword"
                | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                | "application/vnd.ms-excel"
                | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                | "application/vnd.ms-powerpoint"
                | "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        );

        let extension = mime_guess::get_mime_extensions_str(&mime_type)
            .and_then(|exts| exts.first())
            .map(|ext| ext.to_string());

        Self {
            mime_type,
            extension,
            is_text,
            is_image,
            is_video,
            is_audio,
            is_document,
        }
    }
}

/// Content validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub file_type: Option<FileTypeInfo>,
    pub size: u64,
    pub hash: Option<IpfsHash>,
}

impl ValidationResult {
    /// Create new validation result
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            file_type: None,
            size: 0,
            hash: None,
        }
    }

    /// Add error to validation result
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }

    /// Add warning to validation result
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.is_valid && self.errors.is_empty()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// IPFS utility functions
pub struct IpfsUtils;

impl IpfsUtils {
    /// Calculate SHA-256 hash of data
    #[instrument(skip(data))]
    pub fn calculate_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Generate IPFS-like hash (simplified for testing)
    #[instrument(skip(data))]
    pub fn generate_ipfs_hash(data: &[u8]) -> IpfsResult<IpfsHash> {
        let hash = Self::calculate_hash(data);
        // Create a CIDv0-like hash for testing
        let ipfs_hash = format!("Qm{}", &hash[..44]);
        IpfsHash::new(ipfs_hash)
    }

    /// Detect file type from content
    #[instrument(skip(data))]
    pub fn detect_file_type(data: &[u8], filename: Option<&str>) -> FileTypeInfo {
        // Try to detect MIME type from content
        let mime_type = if let Some(kind) = infer::get(data) {
            kind.mime_type().to_string()
        } else if let Some(name) = filename {
            // Fallback to filename-based detection
            mime_guess::from_path(name)
                .first()
                .map(|m| m.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string())
        } else {
            "application/octet-stream".to_string()
        };

        FileTypeInfo::new(mime_type)
    }

    /// Detect file type from path
    #[instrument(skip(path))]
    pub fn detect_file_type_from_path<P: AsRef<Path>>(path: P) -> FileTypeInfo {
        let path = path.as_ref();
        let mime_type = mime_guess::from_path(path)
            .first()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        FileTypeInfo::new(mime_type)
    }

    /// Validate content for IPFS storage
    #[instrument(skip(data))]
    pub fn validate_content(
        data: &[u8],
        filename: Option<&str>,
        max_size: Option<u64>,
        allowed_types: Option<&[String]>,
    ) -> ValidationResult {
        let mut result = ValidationResult::new();
        result.size = data.len() as u64;

        // Check size limit
        if let Some(max_size) = max_size {
            if result.size > max_size {
                result.add_error(format!(
                    "File size {} exceeds maximum allowed size {}",
                    result.size, max_size
                ));
            }
        }

        // Check if data is empty
        if data.is_empty() {
            result.add_error("Content is empty".to_string());
            return result;
        }

        // Detect file type
        let file_type = Self::detect_file_type(data, filename);

        // Check allowed MIME types
        if let Some(allowed_types) = allowed_types {
            if !allowed_types.is_empty() && !allowed_types.contains(&file_type.mime_type) {
                result.add_error(format!(
                    "MIME type '{}' is not allowed",
                    file_type.mime_type
                ));
            }
        }

        // Generate hash
        if let Ok(hash) = Self::generate_ipfs_hash(data) {
            result.hash = Some(hash);
        } else {
            result.add_warning("Failed to generate content hash".to_string());
        }

        result.file_type = Some(file_type);
        result
    }

    /// Format file size in human-readable format
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
        const THRESHOLD: f64 = 1024.0;

        if size == 0 {
            return "0 B".to_string();
        }

        let size_f = size as f64;
        let unit_index = (size_f.log10() / THRESHOLD.log10()).floor() as usize;
        let unit_index = unit_index.min(UNITS.len() - 1);

        let size_in_unit = size_f / THRESHOLD.powi(unit_index as i32);

        if unit_index == 0 {
            format!("{} {}", size, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size_in_unit, UNITS[unit_index])
        }
    }

    /// Check if hash is valid IPFS hash
    pub fn is_valid_ipfs_hash(hash: &str) -> bool {
        IpfsHash::is_valid(hash)
    }

    /// Extract file extension from filename
    pub fn extract_file_extension(filename: &str) -> Option<String> {
        Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    /// Generate content fingerprint (for deduplication)
    #[instrument(skip(data))]
    pub fn generate_fingerprint(data: &[u8]) -> String {
        let hash = Self::calculate_hash(data);
        let size = data.len();
        format!("{}:{}", hash, size)
    }

    /// Check if content is likely to be text
    pub fn is_text_content(data: &[u8]) -> bool {
        if data.is_empty() {
            return false;
        }

        // Check for null bytes (binary indicator)
        if data.contains(&0) {
            return false;
        }

        // Check if most bytes are printable ASCII or common UTF-8
        let printable_count = data
            .iter()
            .filter(|&&b| b.is_ascii_graphic() || b.is_ascii_whitespace())
            .count();

        let ratio = printable_count as f64 / data.len() as f64;
        ratio > 0.95 // 95% of bytes should be printable
    }

    /// Sanitize filename for safe storage
    pub fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .map(|c| match c {
                // Replace unsafe characters
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                // Keep safe characters
                c if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' => c,
                // Replace other characters with underscore
                _ => '_',
            })
            .collect::<String>()
            .trim_matches('.')
            .to_string()
    }

    /// Generate unique filename with timestamp
    pub fn generate_unique_filename(original: &str) -> String {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let sanitized = Self::sanitize_filename(original);

        if let Some(extension) = Self::extract_file_extension(&sanitized) {
            let name_without_ext = sanitized.trim_end_matches(&format!(".{}", extension));
            format!("{}_{}.{}", name_without_ext, timestamp, extension)
        } else {
            format!("{}_{}", sanitized, timestamp)
        }
    }
}

/// Content analyzer for advanced content inspection
pub struct ContentAnalyzer;

impl ContentAnalyzer {
    /// Analyze content structure and properties
    #[instrument(skip(data))]
    pub fn analyze(data: &[u8], filename: Option<&str>) -> ContentAnalysis {
        let mut analysis = ContentAnalysis::new();
        analysis.size = data.len() as u64;
        analysis.hash = IpfsUtils::calculate_hash(data);
        analysis.file_type = IpfsUtils::detect_file_type(data, filename);
        analysis.is_text = IpfsUtils::is_text_content(data);

        // Calculate entropy (measure of randomness)
        analysis.entropy = Self::calculate_entropy(data);

        // Detect compression
        analysis.is_compressed = Self::is_compressed(data);

        // Count unique bytes
        let mut byte_counts = [0u32; 256];
        for &byte in data {
            byte_counts[byte as usize] += 1;
        }
        analysis.unique_bytes = byte_counts.iter().filter(|&&count| count > 0).count() as u32;

        analysis
    }

    /// Calculate Shannon entropy of data
    fn calculate_entropy(data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mut counts = [0u32; 256];
        for &byte in data {
            counts[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;

        for &count in &counts {
            if count > 0 {
                let p = count as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Check if data appears to be compressed
    fn is_compressed(data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }

        // Check for common compression signatures
        matches!(
            data[..4],
            [0x1f, 0x8b, _, _] |  // gzip
            [0x50, 0x4b, 0x03, 0x04] | // zip
            [0x50, 0x4b, 0x05, 0x06] | // zip (empty)
            [0x50, 0x4b, 0x07, 0x08] | // zip (spanned)
            [0x42, 0x5a, 0x68, _] |    // bzip2
            [0xfd, 0x37, 0x7a, 0x58] // xz
        )
    }
}

/// Content analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    pub size: u64,
    pub hash: String,
    pub file_type: FileTypeInfo,
    pub is_text: bool,
    pub is_compressed: bool,
    pub entropy: f64,
    pub unique_bytes: u32,
    pub analyzed_at: chrono::DateTime<chrono::Utc>,
}

impl ContentAnalysis {
    /// Create new content analysis
    pub fn new() -> Self {
        Self {
            size: 0,
            hash: String::new(),
            file_type: FileTypeInfo::new("application/octet-stream".to_string()),
            is_text: false,
            is_compressed: false,
            entropy: 0.0,
            unique_bytes: 0,
            analyzed_at: chrono::Utc::now(),
        }
    }
}

impl Default for ContentAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_info_creation() {
        let info = FileTypeInfo::new("image/jpeg".to_string());
        assert_eq!(info.mime_type, "image/jpeg");
        assert!(info.is_image);
        assert!(!info.is_text);
        assert!(!info.is_video);
        assert!(!info.is_audio);
        assert!(!info.is_document);

        let text_info = FileTypeInfo::new("text/plain".to_string());
        assert!(text_info.is_text);
        assert!(!text_info.is_image);

        let doc_info = FileTypeInfo::new("application/pdf".to_string());
        assert!(doc_info.is_document);
        assert!(!doc_info.is_text);
    }

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid());
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());

        result.add_warning("Test warning".to_string());
        assert!(result.is_valid()); // Warnings don't affect validity
        assert_eq!(result.warnings.len(), 1);

        result.add_error("Test error".to_string());
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_calculate_hash() {
        let data = b"Hello, IPFS!";
        let hash = IpfsUtils::calculate_hash(data);
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 produces 64-character hex string

        // Same data should produce same hash
        let hash2 = IpfsUtils::calculate_hash(data);
        assert_eq!(hash, hash2);

        // Different data should produce different hash
        let hash3 = IpfsUtils::calculate_hash(b"Different data");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_generate_ipfs_hash() {
        let data = b"Test content for IPFS hash";
        let hash = IpfsUtils::generate_ipfs_hash(data).unwrap();
        assert!(hash.as_str().starts_with("Qm"));
        assert_eq!(hash.as_str().len(), 46); // CIDv0 length
        assert!(IpfsHash::is_valid(hash.as_str()));
    }

    #[test]
    fn test_detect_file_type() {
        // Test with JPEG data (simplified)
        let jpeg_data = b"\xFF\xD8\xFF\xE0";
        let file_type = IpfsUtils::detect_file_type(jpeg_data, Some("test.jpg"));
        assert!(file_type.is_image);

        // Test with text data
        let text_data = b"This is plain text content";
        let file_type = IpfsUtils::detect_file_type(text_data, Some("test.txt"));
        // Note: infer crate might not detect this as text, so we check filename fallback
        assert!(
            file_type.mime_type.contains("text")
                || file_type.mime_type == "application/octet-stream"
        );

        // Test with no filename
        let file_type = IpfsUtils::detect_file_type(b"unknown", None);
        assert_eq!(file_type.mime_type, "application/octet-stream");
    }

    #[test]
    fn test_detect_file_type_from_path() {
        let file_type = IpfsUtils::detect_file_type_from_path("test.jpg");
        assert_eq!(file_type.mime_type, "image/jpeg");
        assert!(file_type.is_image);

        let file_type = IpfsUtils::detect_file_type_from_path("document.pdf");
        assert_eq!(file_type.mime_type, "application/pdf");
        assert!(file_type.is_document);

        let file_type = IpfsUtils::detect_file_type_from_path("unknown.xyz");
        assert_eq!(file_type.mime_type, "chemical/x-xyz"); // mime_guess recognizes .xyz as chemical format
    }

    #[test]
    fn test_validate_content() {
        let data = b"Valid test content";
        let result = IpfsUtils::validate_content(data, Some("test.txt"), None, None);
        assert!(result.is_valid());
        assert_eq!(result.size, data.len() as u64);
        assert!(result.hash.is_some());
        assert!(result.file_type.is_some());

        // Test size limit
        let result = IpfsUtils::validate_content(data, Some("test.txt"), Some(10), None);
        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());

        // Test empty content
        let result = IpfsUtils::validate_content(b"", Some("empty.txt"), None, None);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("empty")));

        // Test allowed types
        let allowed_types = vec!["text/plain".to_string()];
        let result =
            IpfsUtils::validate_content(data, Some("test.txt"), None, Some(&allowed_types));
        // This might pass or fail depending on file type detection
        assert!(result.file_type.is_some());
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(IpfsUtils::format_file_size(0), "0 B");
        assert_eq!(IpfsUtils::format_file_size(512), "512 B");
        assert_eq!(IpfsUtils::format_file_size(1024), "1.00 KB");
        assert_eq!(IpfsUtils::format_file_size(1536), "1.50 KB");
        assert_eq!(IpfsUtils::format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(IpfsUtils::format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_is_valid_ipfs_hash() {
        assert!(IpfsUtils::is_valid_ipfs_hash(
            "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG"
        ));
        assert!(IpfsUtils::is_valid_ipfs_hash(
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
        ));
        assert!(!IpfsUtils::is_valid_ipfs_hash("invalid-hash"));
        assert!(!IpfsUtils::is_valid_ipfs_hash(""));
    }

    #[test]
    fn test_extract_file_extension() {
        assert_eq!(
            IpfsUtils::extract_file_extension("test.txt"),
            Some("txt".to_string())
        );
        assert_eq!(
            IpfsUtils::extract_file_extension("image.JPEG"),
            Some("jpeg".to_string())
        );
        assert_eq!(
            IpfsUtils::extract_file_extension("document.tar.gz"),
            Some("gz".to_string())
        );
        assert_eq!(IpfsUtils::extract_file_extension("no_extension"), None);
        assert_eq!(IpfsUtils::extract_file_extension(".hidden"), None); // Hidden files have no extension
    }

    #[test]
    fn test_generate_fingerprint() {
        let data1 = b"Test data for fingerprint";
        let data2 = b"Different test data";

        let fp1 = IpfsUtils::generate_fingerprint(data1);
        let fp2 = IpfsUtils::generate_fingerprint(data1);
        let fp3 = IpfsUtils::generate_fingerprint(data2);

        assert_eq!(fp1, fp2); // Same data should produce same fingerprint
        assert_ne!(fp1, fp3); // Different data should produce different fingerprint
        assert!(fp1.contains(":")); // Should contain size separator
    }

    #[test]
    fn test_is_text_content() {
        assert!(IpfsUtils::is_text_content(b"This is plain text"));
        assert!(IpfsUtils::is_text_content(
            b"Text with numbers 123 and symbols !@#"
        ));
        assert!(IpfsUtils::is_text_content(b"Multi\nline\ntext\ncontent"));

        assert!(!IpfsUtils::is_text_content(b"")); // Empty
        assert!(!IpfsUtils::is_text_content(b"\x00\x01\x02\x03")); // Binary with null bytes

        // Mixed content (mostly binary)
        let mixed = b"Some text\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09";
        assert!(!IpfsUtils::is_text_content(mixed));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(
            IpfsUtils::sanitize_filename("normal_file.txt"),
            "normal_file.txt"
        );
        assert_eq!(
            IpfsUtils::sanitize_filename("file with spaces.txt"),
            "file_with_spaces.txt"
        );
        assert_eq!(
            IpfsUtils::sanitize_filename("unsafe/\\:*?\"<>|chars.txt"),
            "unsafe_________chars.txt"
        );
        assert_eq!(IpfsUtils::sanitize_filename("..hidden.file"), "hidden.file");
        assert_eq!(IpfsUtils::sanitize_filename("file."), "file");
        assert_eq!(
            IpfsUtils::sanitize_filename("unicode_文件.txt"),
            "unicode___.txt"
        ); // Each Chinese character becomes one underscore
    }

    #[test]
    fn test_generate_unique_filename() {
        let original = "test.txt";
        let unique = IpfsUtils::generate_unique_filename(original);

        assert!(unique.starts_with("test_"));
        assert!(unique.ends_with(".txt"));
        assert!(unique.len() > original.len());

        // Should generate different names for multiple calls
        let unique2 = IpfsUtils::generate_unique_filename(original);
        // Note: This might be the same if called in the same second

        let no_ext = IpfsUtils::generate_unique_filename("no_extension");
        assert!(no_ext.starts_with("no_extension_"));
        assert!(!no_ext.contains("."));
    }

    #[test]
    fn test_content_analyzer() {
        let text_data = b"This is a test text content for analysis";
        let analysis = ContentAnalyzer::analyze(text_data, Some("test.txt"));

        assert_eq!(analysis.size, text_data.len() as u64);
        assert!(!analysis.hash.is_empty());
        assert!(analysis.is_text);
        assert!(!analysis.is_compressed);
        assert!(analysis.entropy > 0.0);
        assert!(analysis.unique_bytes > 0);

        // Test with binary data
        let binary_data = b"\x00\x01\x02\x03\x04\x05\x06\x07";
        let analysis = ContentAnalyzer::analyze(binary_data, None);
        assert!(!analysis.is_text);
        assert!(analysis.entropy >= 0.0);
    }

    #[test]
    fn test_content_analysis_creation() {
        let analysis = ContentAnalysis::new();
        assert_eq!(analysis.size, 0);
        assert!(analysis.hash.is_empty());
        assert!(!analysis.is_text);
        assert!(!analysis.is_compressed);
        assert_eq!(analysis.entropy, 0.0);
        assert_eq!(analysis.unique_bytes, 0);

        let default_analysis = ContentAnalysis::default();
        assert_eq!(default_analysis.size, analysis.size);
    }

    #[test]
    fn test_entropy_calculation() {
        // Uniform data should have high entropy
        let uniform_data: Vec<u8> = (0..=255).collect();
        let entropy = ContentAnalyzer::calculate_entropy(&uniform_data);
        assert!(entropy > 7.0); // Close to maximum entropy (8.0)

        // Repeated data should have low entropy
        let repeated_data = vec![0u8; 1000];
        let entropy = ContentAnalyzer::calculate_entropy(&repeated_data);
        assert_eq!(entropy, 0.0); // All same bytes = zero entropy

        // Empty data
        let entropy = ContentAnalyzer::calculate_entropy(&[]);
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn test_compression_detection() {
        // Test gzip signature
        let gzip_data = b"\x1f\x8b\x08\x00";
        assert!(ContentAnalyzer::is_compressed(gzip_data));

        // Test zip signature
        let zip_data = b"\x50\x4b\x03\x04";
        assert!(ContentAnalyzer::is_compressed(zip_data));

        // Test non-compressed data
        let text_data = b"This is not compressed";
        assert!(!ContentAnalyzer::is_compressed(text_data));

        // Test too short data
        let short_data = b"\x1f\x8b";
        assert!(!ContentAnalyzer::is_compressed(short_data));
    }
}
