use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientConfig {
    /// URI client connect to
    pub uri: String,
    /// domain the cert belongs to
    pub domain: String,
    /// CA cert
    pub ca_cert: String,
    /// client cert
    pub client_cert: Option<CertConfig>,
    /// pool size
    pub pool_size: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CertConfig {
    /// client cert
    pub cert: String,
    /// client cert key
    pub sk: String,
}

impl ClientConfig {
    pub fn new(
        uri: &str,
        domain: &str,
        ca_cert: &str,
        client_cert: Option<CertConfig>,
        size: u8,
    ) -> Self {
        let pool_size = min(16, max(2, size));
        Self {
            uri: uri.to_owned(),
            domain: domain.to_owned(),
            ca_cert: ca_cert.to_owned(),
            client_cert,
            pool_size,
        }
    }
}

impl CertConfig {
    pub fn new(cert: &str, sk: &str) -> Self {
        Self {
            cert: cert.to_owned(),
            sk: sk.to_owned(),
        }
    }
}
