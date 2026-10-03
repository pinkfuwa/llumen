//! URL validation utilities including private IP checking.

use anyhow::Result;
use std::net::IpAddr;

/// Check if an IP address is in a private network range
pub fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(addr) => {
            // RFC 1918 private IPv4 ranges
            addr.is_private()
                || addr.is_loopback()
                || addr.is_link_local()
                || addr.is_broadcast()
                || addr.is_documentation()
                || addr.is_multicast()
                || addr.is_unspecified()
        }
        IpAddr::V6(addr) => {
            // RFC 4193 private IPv6 ranges
            addr.is_loopback()
                || addr.is_unspecified()
                || (addr.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7
                || addr.is_unicast_link_local()
                || addr.is_multicast()
                || addr
                    .to_ipv4_mapped()
                    .is_some_and(|mapped| is_private_ip(&IpAddr::V4(mapped)))
        }
    }
}

/// Validate a URL and ensure it doesn't point to private IP addresses
pub async fn validate_url(url: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url)?;
    if !matches!(parsed.scheme(), "http" | "https") {
        anyhow::bail!("Only HTTP and HTTPS URLs are allowed");
    }

    // Check if the host is an IP address
    if let Some(host) = parsed.host_str() {
        // Try to parse as IP address
        if let Ok(ip) = host.parse::<IpAddr>() {
            if is_private_ip(&ip) {
                anyhow::bail!("Access to private IP addresses is not allowed");
            }
        } else {
            // Resolve hostname to check IP addresses
            match tokio::net::lookup_host(format!("{}:{}", host, parsed.port().unwrap_or(80))).await
            {
                Ok(addrs) => {
                    for addr in addrs {
                        if is_private_ip(&addr.ip()) {
                            anyhow::bail!(
                                "Hostname resolves to private IP address, access not allowed"
                            );
                        }
                    }
                }
                Err(e) => {
                    anyhow::bail!("Failed to resolve hostname: {}", e);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_private_ip_detection() {
        // IPv4 private addresses
        assert!(is_private_ip(&"192.168.1.1".parse().unwrap()));
        assert!(is_private_ip(&"10.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"172.16.0.1".parse().unwrap()));
        assert!(is_private_ip(&"127.0.0.1".parse().unwrap()));

        // IPv4 public addresses
        assert!(!is_private_ip(&"8.8.8.8".parse().unwrap()));
        assert!(!is_private_ip(&"1.1.1.1".parse().unwrap()));

        // IPv6 private addresses
        assert!(is_private_ip(&"::1".parse().unwrap()));
        assert!(is_private_ip(&"fc00::1".parse().unwrap()));

        // IPv6 public addresses
        assert!(!is_private_ip(&"2001:4860:4860::8888".parse().unwrap()));
    }

    #[test]
    fn rejects_non_public_ip_ranges() {
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(224, 0, 0, 1))));
        assert!(is_private_ip(&IpAddr::V6(Ipv6Addr::new(
            0xfe80, 0, 0, 0, 0, 0, 0, 1
        ))));
        assert!(is_private_ip(&IpAddr::V6(Ipv6Addr::new(
            0, 0, 0, 0, 0, 0xffff, 0x7f00, 1
        ))));
        assert!(!is_private_ip(&IpAddr::V6(Ipv6Addr::new(
            0, 0, 0, 0, 0, 0xffff, 0x0808, 0x0808
        ))));
    }

    #[tokio::test]
    async fn rejects_non_http_urls() {
        assert!(validate_url("file:///etc/passwd").await.is_err());
        assert!(validate_url("ftp://8.8.8.8/data").await.is_err());
    }

    #[tokio::test]
    async fn rejects_ipv4_mapped_loopback_urls() {
        assert!(validate_url("http://[::ffff:127.0.0.1]/").await.is_err());
    }

    #[tokio::test]
    async fn rejects_ipv6_link_local_urls() {
        assert!(validate_url("http://[fe80::1]/").await.is_err());
    }

    #[tokio::test]
    async fn accepts_public_http_urls() {
        assert!(validate_url("http://8.8.8.8/").await.is_ok());
        assert!(
            validate_url("https://[2001:4860:4860::8888]/")
                .await
                .is_ok()
        );
    }
}
