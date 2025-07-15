//! salt_atproto_core    
//! core tools

#![feature(iter_intersperse)]
use thiserror::Error;

pub mod atproto;
pub mod dns;

pub use atproto::{AtProtoClient, atproto_client};
pub use dns::{DnsClient, dns_client};

/// Our Error Type
#[derive(Clone, Debug, Error)]
pub enum SaltError {
    #[error("atproto error")]
    AtProtoError,
    #[error("atproto {0} not found")]
    AtProtoNotFoundError(String),
    #[error("dns resolution error")]
    DNSError(#[from] hickory_client::ClientError),
    #[error("no TXT record found at {0}")]
    MissingTXTError(String),
    #[error("invalid DID {0}")]
    DIDError(String),
    #[error("invalid identifier {0}")]
    IdentifierError(String),
}
