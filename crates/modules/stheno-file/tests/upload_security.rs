use stheno_file::UploadPipeline;

#[test]
fn test_upload_pipeline_magic_byte_sniffing_and_svg_rejection() {
    let cdn_origin = "https://cdn.sthenostore.io";

    // 1. Valid PNG
    let png_bytes = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
    let res = UploadPipeline::process_upload(&png_bytes, "image/png", cdn_origin);
    assert!(res.is_ok());
    let url = res.unwrap();
    assert!(url.starts_with("https://cdn.sthenostore.io/uploads/"));
    assert!(url.ends_with(".png"));

    // 2. Valid JPEG
    let jpeg_bytes = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46];
    let res_jpg = UploadPipeline::process_upload(&jpeg_bytes, "image/jpeg", cdn_origin);
    assert!(res_jpg.is_ok());
    assert!(res_jpg.unwrap().ends_with(".jpg"));

    // 3. Reject SVG disguised as image/svg+xml or image/png
    let svg_bytes = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script>alert(1)</script></svg>";
    let res_svg = UploadPipeline::process_upload(svg_bytes, "image/svg+xml", cdn_origin);
    assert!(res_svg.is_err());
    let err_str = res_svg.unwrap_err().to_string();
    assert!(err_str.contains("svg_disallowed"));

    // 4. Reject MIME mismatch (fake PNG header with JPEG payload)
    let mismatch = UploadPipeline::process_upload(&jpeg_bytes, "image/png", cdn_origin);
    assert!(mismatch.is_err());
    assert!(mismatch.unwrap_err().to_string().contains("mime_mismatch"));
}
