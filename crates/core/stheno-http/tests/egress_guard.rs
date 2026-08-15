use stheno_http::EgressGuard;

#[test]
fn test_egress_guard_scheme_ip_denylist_and_redirects() {
    // 1. Allowed valid external URLs
    assert!(EgressGuard::validate_url("https://api.stripe.com/v1/charges").is_ok());
    assert!(EgressGuard::validate_url("http://cdn.example.com/images/1.jpg").is_ok());

    // 2. Reject disallowed schemes (e.g. file://, ftp://, gopher://)
    let file_res = EgressGuard::validate_url("file:///etc/passwd");
    assert!(file_res.is_err());
    assert!(file_res
        .unwrap_err()
        .to_string()
        .contains("disallowed_scheme"));

    let gopher_res = EgressGuard::validate_url("gopher://127.0.0.1:70/");
    assert!(gopher_res.is_err());

    // 3. Reject Link-Local Cloud Metadata IP: 169.254.169.254 (169.254.0.0/16)
    let meta_res = EgressGuard::validate_url("http://169.254.169.254/latest/meta-data/");
    assert!(meta_res.is_err());
    assert!(meta_res
        .unwrap_err()
        .to_string()
        .contains("egress_cloud_metadata_prohibited"));

    // 4. Reject Loopback IP: 127.0.0.1
    let loopback_res = EgressGuard::validate_url("http://127.0.0.1:8080/admin");
    assert!(loopback_res.is_err());
    assert!(loopback_res
        .unwrap_err()
        .to_string()
        .contains("egress_loopback_prohibited"));

    // 5. Reject Private IPv4: 10.0.0.1, 192.168.1.1, 172.16.0.5
    let priv_res = EgressGuard::validate_url("http://192.168.1.1/router");
    assert!(priv_res.is_err());
    assert!(priv_res
        .unwrap_err()
        .to_string()
        .contains("egress_private_ip_prohibited"));

    // 6. Redirect hop re-checking
    let start_url = EgressGuard::validate_url("https://example.com/landing").unwrap();
    // Redirect to legitimate path on same host
    let safe_redirect = EgressGuard::validate_redirect_hop(&start_url, "/checkout");
    assert!(safe_redirect.is_ok());

    // Malicious redirect trying to hop to cloud metadata
    let evil_redirect = EgressGuard::validate_redirect_hop(&start_url, "http://169.254.169.254/");
    assert!(evil_redirect.is_err());
}
