use reqwest::{Certificate, Client, ClientBuilder};
use std::fs;
use std::path::Path;

/// Build a reqwest Client with optional custom certificate support.
///
/// When the `dev` feature is enabled, this function will check the
/// `LLUMEN_CUSTOM_CERT_PATH` environment variable. If set, it will load
/// the certificate from the specified path and add it as a root certificate.
///
/// This is useful for testing/benchmarking with self-signed certificates or
/// custom CAs.
///
/// # Environment Variables
///
/// - `LLUMEN_CUSTOM_CERT_PATH`: Path to a PEM or DER encoded certificate file
///   (only available when compiled with the `dev` feature)
///
/// # Examples
///
/// ```no_run
/// // Build a client with default settings
/// let client = build_client();
///
/// // Build a client with custom builder settings
/// let client = build_client_with(|builder| {
///     builder.timeout(std::time::Duration::from_secs(30))
/// });
/// ```
pub fn build_client() -> Client {
    build_client_with(|builder| builder)
}

/// Build a reqwest Client with custom configuration and optional certificate support.
///
/// This allows you to customize the ClientBuilder before it's built.
///
/// # Arguments
///
/// * `configure` - A function that takes a ClientBuilder and returns a configured ClientBuilder
///
/// # Examples
///
/// ```no_run
/// let client = build_client_with(|builder| {
///     builder
///         .timeout(std::time::Duration::from_secs(30))
///         .user_agent("my-app/1.0")
/// });
/// ```
pub fn build_client_with<F>(configure: F) -> Client
where
    F: FnOnce(ClientBuilder) -> ClientBuilder,
{
    let mut builder = Client::builder();

    #[cfg(feature = "dev")]
    {
        if let Ok(cert_path) = std::env::var("LLUMEN_CUSTOM_CERT_PATH") {
            match load_certificate(&cert_path) {
                Ok(cert) => {
                    log::info!("Loaded custom certificate from: {}", cert_path);
                    builder = builder.add_root_certificate(cert);
                }
                Err(e) => {
                    log::error!("Failed to load custom certificate from {}: {}", cert_path, e);
                }
            }
        }
    }

    configure(builder)
        .build()
        .expect("Failed to build HTTP client")
}

/// Load a certificate from a file path.
///
/// Supports both PEM and DER encoded certificates.
/// Will first try to load as PEM, and if that fails, try as DER.
#[cfg(feature = "dev")]
fn load_certificate(path: &str) -> Result<Certificate, Box<dyn std::error::Error>> {
    let cert_path = Path::new(path);
    if !cert_path.exists() {
        return Err(format!("Certificate file not found: {}", path).into());
    }

    let cert_bytes = fs::read(cert_path)?;

    // Try PEM first
    match Certificate::from_pem(&cert_bytes) {
        Ok(cert) => {
            log::debug!("Loaded certificate as PEM format");
            Ok(cert)
        }
        Err(_) => {
            // Try DER format
            match Certificate::from_der(&cert_bytes) {
                Ok(cert) => {
                    log::debug!("Loaded certificate as DER format");
                    Ok(cert)
                }
                Err(e) => Err(format!("Failed to parse certificate as PEM or DER: {}", e).into()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_client_without_custom_cert() {
        // Should build successfully without custom cert
        let client = build_client();
        assert!(client.get("https://example.com").build().is_ok());
    }

    #[test]
    fn test_build_client_with_custom_config() {
        let client = build_client_with(|builder| {
            builder.timeout(std::time::Duration::from_secs(10))
        });
        assert!(client.get("https://example.com").build().is_ok());
    }

    #[cfg(feature = "dev")]
    #[test]
    fn test_load_certificate_nonexistent_file() {
        let result = load_certificate("/nonexistent/path/cert.pem");
        assert!(result.is_err());
    }
}
