//! Egress guard for SSRF protection: scheme check, resolved-IP denylist, and redirect re-check (SECURITY.md §8).

use std::net::{IpAddr, Ipv6Addr};
use stheno_types::{Error, Result};
use url::Url;

pub struct EgressGuard;

impl EgressGuard {
    /// Validates an outbound URL against SSRF vulnerabilities before any network request or redirect hop.
    pub fn validate_url(raw_url: &str) -> Result<Url> {
        let parsed = Url::parse(raw_url).map_err(|e| Error::Invalid {
            code: "invalid_egress_url",
            message: format!("Malformed URL: {e}"),
            field: Some("url".to_string()),
        })?;

        // 1. Scheme check: only http and https allowed
        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return Err(Error::Invalid {
                code: "disallowed_scheme",
                message: format!(
                    "Scheme '{}' is not permitted for outbound requests",
                    parsed.scheme()
                ),
                field: Some("url".to_string()),
            });
        }

        let host = parsed.host_str().ok_or_else(|| Error::Invalid {
            code: "missing_host",
            message: "Outbound URL must contain a valid host".to_string(),
            field: Some("url".to_string()),
        })?;

        // 2. Direct IP parsing check
        if let Ok(ip) = host.parse::<IpAddr>() {
            Self::validate_ip(ip)?;
        }

        Ok(parsed)
    }

    /// Checks if a resolved IP falls into any prohibited address space (denylist).
    pub fn validate_ip(ip: IpAddr) -> Result<()> {
        match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();

                // 127.0.0.0/8 (Loopback)
                if ipv4.is_loopback() || octets[0] == 127 {
                    return Err(Error::Invalid {
                        code: "egress_loopback_prohibited",
                        message: "Outbound request to loopback IP address is prohibited"
                            .to_string(),
                        field: Some("ip".to_string()),
                    });
                }

                // 169.254.0.0/16 (Link-local cloud metadata service)
                if ipv4.is_link_local() || (octets[0] == 169 && octets[1] == 254) {
                    return Err(Error::Invalid {
                        code: "egress_cloud_metadata_prohibited",
                        message: "Outbound request to link-local cloud metadata (169.254.0.0/16) is prohibited".to_string(),
                        field: Some("ip".to_string()),
                    });
                }

                // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16 (Private addresses)
                if ipv4.is_private()
                    || octets[0] == 10
                    || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                    || (octets[0] == 192 && octets[1] == 168)
                {
                    return Err(Error::Invalid {
                        code: "egress_private_ip_prohibited",
                        message: "Outbound request to private network IP address is prohibited"
                            .to_string(),
                        field: Some("ip".to_string()),
                    });
                }

                // 0.0.0.0/8 (Broadcast/this host)
                if octets[0] == 0 {
                    return Err(Error::Invalid {
                        code: "egress_unspecified_prohibited",
                        message: "Outbound request to 0.0.0.0 is prohibited".to_string(),
                        field: Some("ip".to_string()),
                    });
                }
            }
            IpAddr::V6(ipv6) => {
                // ::1 (Loopback)
                if ipv6.is_loopback() || ipv6 == Ipv6Addr::LOCALHOST {
                    return Err(Error::Invalid {
                        code: "egress_loopback_prohibited",
                        message: "Outbound request to IPv6 loopback is prohibited".to_string(),
                        field: Some("ip".to_string()),
                    });
                }

                let segments = ipv6.segments();
                // fe80::/10 (Link-local)
                if (segments[0] & 0xffc0) == 0xfe80 {
                    return Err(Error::Invalid {
                        code: "egress_link_local_prohibited",
                        message: "Outbound request to IPv6 link-local is prohibited".to_string(),
                        field: Some("ip".to_string()),
                    });
                }

                // fc00::/7 (Unique local address)
                if (segments[0] & 0xfe00) == 0xfc00 {
                    return Err(Error::Invalid {
                        code: "egress_private_ip_prohibited",
                        message: "Outbound request to IPv6 unique local address is prohibited"
                            .to_string(),
                        field: Some("ip".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Verifies redirect destination URL on each hop
    pub fn validate_redirect_hop(current_url: &Url, next_location: &str) -> Result<Url> {
        let next_url = current_url
            .join(next_location)
            .map_err(|e| Error::Invalid {
                code: "invalid_redirect_url",
                message: format!("Failed to parse redirect location: {e}"),
                field: Some("redirect_location".to_string()),
            })?;

        Self::validate_url(next_url.as_str())
    }
}
