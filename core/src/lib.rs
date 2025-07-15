#![feature(iter_intersperse)]
use thiserror::Error;

pub mod atproto;
pub mod dns;

pub use atproto::{AtProtoClient, atproto_client};
pub use dns::{DnsClient, dns_client};

#[derive(Clone, Debug, Error)]
pub enum AppError {
    #[error("atproto error")]
    AtProtoError,
    #[error("dns resolution error")]
    DNSError(#[from] hickory_client::ClientError),
    #[error("no TXT record found at {0}")]
    MissingTXTError(String),
    #[error("invalid DID {0}")]
    DIDError(String),
    #[error("invalid identifier {0}")]
    IdentifierError(String),
}
