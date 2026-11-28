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

    /// Extracts image dimensions from PNG, JPEG, or WebP binary payload.
    pub fn sniff_image_dimensions(bytes: &[u8]) -> Result<ImageDimensions> {
        let mime = Self::sniff_mime_type(bytes)?;
        match mime {
            AllowedMimeType::Png => {
                if bytes.len() < 24 {
                    return Err(Error::Invalid {
                        code: "invalid_png",
                        message: "PNG payload too short for IHDR header".to_string(),
                        field: Some("file".to_string()),
                    });
                }
                let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
                let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
                Ok(ImageDimensions { width, height })
            }
            AllowedMimeType::Jpeg => {
                let mut i = 2;
                while i + 8 < bytes.len() {
                    if bytes[i] != 0xFF {
                        i += 1;
                        continue;
                    }
                    let marker = bytes[i + 1];
                    if marker == 0xFF || marker == 0x00 {
                        i += 1;
                        continue;
                    }
                    // SOF markers: SOF0..SOF3, SOF5..SOF7, SOF9..SOF11, SOF13..SOF15
                    if matches!(marker, 0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF) {
                        let height = ((bytes[i + 5] as u32) << 8) | (bytes[i + 6] as u32);
                        let width = ((bytes[i + 7] as u32) << 8) | (bytes[i + 8] as u32);
                        return Ok(ImageDimensions { width, height });
                    }
                    if i + 3 >= bytes.len() {
                        break;
                    }
                    let length = ((bytes[i + 2] as usize) << 8) | (bytes[i + 3] as usize);
                    i += 2 + length;
                }
                Err(Error::Invalid {
                    code: "invalid_jpeg",
                    message: "JPEG SOF segment not found in payload".to_string(),
                    field: Some("file".to_string()),
                })
            }
            AllowedMimeType::Webp => {
                if bytes.len() < 30 {
                    return Err(Error::Invalid {
                        code: "invalid_webp",
                        message: "WebP payload too short".to_string(),
                        field: Some("file".to_string()),
                    });
                }
                let tag = &bytes[12..16];
                if tag == b"VP8 " && bytes.len() >= 30 {
                    let width = (u16::from_le_bytes([bytes[26], bytes[27]]) & 0x3fff) as u32;
                    let height = (u16::from_le_bytes([bytes[28], bytes[29]]) & 0x3fff) as u32;
                    Ok(ImageDimensions { width, height })
                } else if tag == b"VP8L" && bytes.len() >= 25 {
                    let b1 = bytes[21] as u32;
                    let b2 = bytes[22] as u32;
                    let b3 = bytes[23] as u32;
                    let b4 = bytes[24] as u32;
                    let width = 1 + ((b1 | (b2 << 8)) & 0x3fff);
                    let height = 1 + (((b2 >> 6) | (b3 << 2) | (b4 << 10)) & 0x3fff);
                    Ok(ImageDimensions { width, height })
                } else if tag == b"VP8X" && bytes.len() >= 30 {
                    let width = 1
                        + (bytes[24] as u32
                            | ((bytes[25] as u32) << 8)
                            | ((bytes[26] as u32) << 16));
                    let height = 1
                        + (bytes[27] as u32
                            | ((bytes[28] as u32) << 8)
                            | ((bytes[29] as u32) << 16));
                    Ok(ImageDimensions { width, height })
                } else {
                    Err(Error::Invalid {
                        code: "invalid_webp",
                        message: "Unrecognized WebP chunk format".to_string(),
                        field: Some("file".to_string()),
                    })
                }
            }
            AllowedMimeType::Pdf => Err(Error::Invalid {
                code: "unsupported_branding_asset_type",
                message: "PDF documents cannot be used as branding logo or favicon assets"
                    .to_string(),
                field: Some("file".to_string()),
            }),
        }
    }

    /// Validates a branding asset (Logo or Favicon) at the boundary (P26-06).
    ///
    /// Logo constraints:
    /// - Formats: PNG, JPEG, WebP (SVGs strictly disallowed).
    /// - Dimensions: Width and height between 32px and 2500px.
    /// - Max size: 5 MB.
    ///
    /// Favicon constraints:
    /// - Formats: PNG, WebP, JPEG.
    /// - Aspect ratio: Must be 1:1 square.
    /// - Dimensions: Between 16px and 512px.
    /// - Max size: 1 MB.
    pub fn validate_branding_asset(
        bytes: &[u8],
        kind: BrandingAssetKind,
    ) -> Result<ImageDimensions> {
        let max_bytes = match kind {
            BrandingAssetKind::Logo => 5 * 1024 * 1024,
            BrandingAssetKind::Favicon => 1024 * 1024,
        };

        if bytes.len() > max_bytes {
            return Err(Error::Invalid {
                code: "file_too_large",
                message: format!(
                    "Branding asset exceeds maximum allowed size of {max_bytes} bytes"
                ),
                field: Some("file".to_string()),
            });
        }

        let dims = Self::sniff_image_dimensions(bytes)?;

        match kind {
            BrandingAssetKind::Logo => {
                if dims.width < 32 || dims.height < 32 {
                    return Err(Error::Invalid {
                        code: "logo_too_small",
                        message: format!(
                            "Logo dimensions ({}x{}) are below the 32x32 minimum",
                            dims.width, dims.height
                        ),
                        field: Some("dimensions".to_string()),
                    });
                }
                if dims.width > 2500 || dims.height > 2500 {
                    return Err(Error::Invalid {
                        code: "logo_too_large",
                        message: format!(
                            "Logo dimensions ({}x{}) exceed the 2500x2500 maximum",
                            dims.width, dims.height
                        ),
                        field: Some("dimensions".to_string()),
                    });
                }
            }
            BrandingAssetKind::Favicon => {
                if dims.width != dims.height {
                    return Err(Error::Invalid {
                        code: "favicon_not_square",
                        message: format!(
                            "Favicon must have a 1:1 square aspect ratio; got {}x{}",
                            dims.width, dims.height
                        ),
                        field: Some("dimensions".to_string()),
                    });
                }
                if dims.width < 16 || dims.height < 16 {
                    return Err(Error::Invalid {
                        code: "favicon_too_small",
                        message: format!(
                            "Favicon dimensions ({}x{}) are below the 16x16 minimum",
                            dims.width, dims.height
                        ),
                        field: Some("dimensions".to_string()),
                    });
                }
                if dims.width > 512 || dims.height > 512 {
                    return Err(Error::Invalid {
                        code: "favicon_too_large",
                        message: format!(
                            "Favicon dimensions ({}x{}) exceed the 512x512 maximum",
                            dims.width, dims.height
                        ),
                        field: Some("dimensions".to_string()),
                    });
                }
            }
        }

        Ok(dims)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrandingAssetKind {
    Logo,
    Favicon,
}
