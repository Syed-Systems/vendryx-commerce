//! P26-06: Logo and favicon upload type and dimension validation at the boundary.

use stheno_file::{BrandingAssetKind, ImageDimensions, UploadPipeline};

fn make_png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    // PNG signature
    bytes.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    // IHDR chunk length (13 bytes)
    bytes.extend_from_slice(&13u32.to_be_bytes());
    // IHDR
    bytes.extend_from_slice(b"IHDR");
    // Width & Height
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    // Bit depth (8), color type (2: RGB), comp (0), filter (0), interlace (0)
    bytes.extend_from_slice(&[8, 2, 0, 0, 0]);
    // CRC placeholder
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    bytes
}

fn make_jpeg(width: u16, height: u16) -> Vec<u8> {
    let mut bytes = Vec::new();
    // SOI
    bytes.extend_from_slice(&[0xFF, 0xD8]);
    // APP0
    bytes.extend_from_slice(&[
        0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0, 1, 1, 1, 0, 72, 0, 72, 0, 0,
    ]);
    // SOF0 marker
    bytes.extend_from_slice(&[0xFF, 0xC0]);
    // Length: 17 bytes
    bytes.extend_from_slice(&[0x00, 0x11]);
    // Precision: 8
    bytes.push(0x08);
    // Height & Width
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&width.to_be_bytes());
    // Components (3: YCbCr)
    bytes.extend_from_slice(&[0x03, 0x01, 0x11, 0x00, 0x02, 0x11, 0x01, 0x03, 0x11, 0x01]);
    // EOI
    bytes.extend_from_slice(&[0xFF, 0xD9]);
    bytes
}

fn make_webp(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&30u32.to_le_bytes());
    bytes.extend_from_slice(b"WEBP");
    bytes.extend_from_slice(b"VP8X");
    bytes.extend_from_slice(&10u32.to_le_bytes());
    bytes.extend_from_slice(&[0, 0, 0, 0]); // Flags
    let w = width - 1;
    let h = height - 1;
    bytes.push((w & 0xFF) as u8);
    bytes.push(((w >> 8) & 0xFF) as u8);
    bytes.push(((w >> 16) & 0xFF) as u8);
    bytes.push((h & 0xFF) as u8);
    bytes.push(((h >> 8) & 0xFF) as u8);
    bytes.push(((h >> 16) & 0xFF) as u8);
    bytes
}

#[test]
fn valid_logo_png_accepted() {
    let png = make_png(400, 100);
    let dims = UploadPipeline::validate_branding_asset(&png, BrandingAssetKind::Logo)
        .expect("valid logo png");
    assert_eq!(
        dims,
        ImageDimensions {
            width: 400,
            height: 100
        }
    );
}

#[test]
fn valid_logo_jpeg_accepted() {
    let jpeg = make_jpeg(800, 600);
    let dims = UploadPipeline::validate_branding_asset(&jpeg, BrandingAssetKind::Logo)
        .expect("valid logo jpeg");
    assert_eq!(
        dims,
        ImageDimensions {
            width: 800,
            height: 600
        }
    );
}

#[test]
fn valid_logo_webp_accepted() {
    let webp = make_webp(500, 500);
    let dims = UploadPipeline::validate_branding_asset(&webp, BrandingAssetKind::Logo)
        .expect("valid logo webp");
    assert_eq!(
        dims,
        ImageDimensions {
            width: 500,
            height: 500
        }
    );
}

#[test]
fn logo_below_minimum_dimensions_rejected() {
    let tiny_png = make_png(24, 24);
    let err =
        UploadPipeline::validate_branding_asset(&tiny_png, BrandingAssetKind::Logo).unwrap_err();
    assert!(err.to_string().contains("logo_too_small"));
}

#[test]
fn logo_above_maximum_dimensions_rejected() {
    let huge_png = make_png(3000, 2000);
    let err =
        UploadPipeline::validate_branding_asset(&huge_png, BrandingAssetKind::Logo).unwrap_err();
    assert!(err.to_string().contains("logo_too_large"));
}

#[test]
fn svg_as_logo_strictly_rejected() {
    let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><path d=\"M0 0\"/></svg>";
    let err = UploadPipeline::validate_branding_asset(svg, BrandingAssetKind::Logo).unwrap_err();
    assert!(err.to_string().contains("svg_disallowed"));
}

#[test]
fn pdf_as_logo_rejected() {
    let pdf = b"%PDF-1.4 sample file content";
    let err = UploadPipeline::validate_branding_asset(pdf, BrandingAssetKind::Logo).unwrap_err();
    assert!(err.to_string().contains("unsupported_branding_asset_type"));
}

#[test]
fn valid_favicon_accepted() {
    let favicon_png = make_png(64, 64);
    let dims = UploadPipeline::validate_branding_asset(&favicon_png, BrandingAssetKind::Favicon)
        .expect("valid favicon");
    assert_eq!(
        dims,
        ImageDimensions {
            width: 64,
            height: 64
        }
    );
}

#[test]
fn non_square_favicon_rejected() {
    let rect_png = make_png(64, 32);
    let err =
        UploadPipeline::validate_branding_asset(&rect_png, BrandingAssetKind::Favicon).unwrap_err();
    assert!(err.to_string().contains("favicon_not_square"));
}

#[test]
fn favicon_below_minimum_rejected() {
    let tiny_favicon = make_png(12, 12);
    let err = UploadPipeline::validate_branding_asset(&tiny_favicon, BrandingAssetKind::Favicon)
        .unwrap_err();
    assert!(err.to_string().contains("favicon_too_small"));
}

#[test]
fn favicon_above_maximum_rejected() {
    let huge_favicon = make_png(1024, 1024);
    let err = UploadPipeline::validate_branding_asset(&huge_favicon, BrandingAssetKind::Favicon)
        .unwrap_err();
    assert!(err.to_string().contains("favicon_too_large"));
}
