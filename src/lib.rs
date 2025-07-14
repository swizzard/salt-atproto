#![feature(iter_intersperse)]
use thiserror::Error;

pub mod dns;

#[derive(Clone, Debug, Error)]
pub enum AppError {
    #[error("dns resolution error")]
    DNSError(#[from] hickory_client::ClientError),
    #[error("no TXT record found at {0}")]
    MissingTXTError(String),
    #[error("invalid DID {0}")]
    DIDError(String),
}
