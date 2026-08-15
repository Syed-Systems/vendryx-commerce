//! `stheno-file`: Secure file upload pipeline with magic-byte sniffing, SVG disallowance, and CDN origin isolation (SECURITY.md §6).

pub mod dto;
pub(crate) mod models;
pub mod provider;
pub mod service;

use serde::{Deserialize, Serialize};
use stheno_types::{Error, Result};

pub use dto::*;
pub use provider::{FileProvider, FileUploadResult, PresignedUrlResult};
pub use service::FileService;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllowedMimeType {
    Jpeg,
    Png,
    Webp,
    Pdf,
}

impl AllowedMimeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Webp => "image/webp",
            Self::Pdf => "application/pdf",
        }
    }
}

pub struct UploadPipeline;

impl UploadPipeline {
    /// Sniffs magic bytes to determine true binary format.
    /// Unconditionally rejects SVGs (stored XSS risk) and un-allowed formats.
    pub fn sniff_mime_type(bytes: &[u8]) -> Result<AllowedMimeType> {
        if bytes.len() < 4 {
            return Err(Error::Invalid {
                code: "invalid_file_payload",
                message: "File payload too small to determine format".to_string(),
                field: Some("file".to_string()),
            });
        }

        // SVG sniffing (reject immediately)
        if bytes.starts_with(b"<?xml") || bytes.starts_with(b"<svg") {
            return Err(Error::Invalid {
                code: "svg_disallowed",
                message: "SVG files are strictly disallowed due to XSS vulnerability risks"
                    .to_string(),
                field: Some("file".to_string()),
            });
        }

        // JPEG: FF D8 FF
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Ok(AllowedMimeType::Jpeg);
        }

        // PNG: 89 50 4E 47 0D 0A 1A 0A
        if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return Ok(AllowedMimeType::Png);
        }

        // WebP: RIFF ... WEBP
        if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
            return Ok(AllowedMimeType::Webp);
        }

        // PDF: %PDF
        if bytes.starts_with(b"%PDF") {
            return Ok(AllowedMimeType::Pdf);
        }

        Err(Error::Invalid {
            code: "unsupported_media_type",
            message: "File format is not allowed. Only JPEG, PNG, WebP, and PDF are supported."
                .to_string(),
            field: Some("file".to_string()),
        })
    }

    /// Validates upload file and produces separate CDN origin URL
    pub fn process_upload(bytes: &[u8], claimed_mime: &str, cdn_base_url: &str) -> Result<String> {
        let detected = Self::sniff_mime_type(bytes)?;
        if detected.as_str() != claimed_mime && !claimed_mime.is_empty() {
            return Err(Error::Invalid {
                code: "mime_mismatch",
                message: format!(
                    "Claimed MIME type '{}' does not match sniffed binary MIME type '{}'",
                    claimed_mime,
                    detected.as_str()
                ),
                field: Some("content_type".to_string()),
            });
        }

        let file_id = ulid::Ulid::new();
        let extension = match detected {
            AllowedMimeType::Jpeg => "jpg",
            AllowedMimeType::Png => "png",
            AllowedMimeType::Webp => "webp",
            AllowedMimeType::Pdf => "pdf",
        };

        // Generates isolated CDN URL (separate origin)
        Ok(format!(
            "{}/uploads/{}.{}",
            cdn_base_url.trim_end_matches('/'),
            file_id,
            extension
        ))
    }
}
