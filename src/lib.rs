#![feature(iter_intersperse)]
use atrium_api::types::string::{Did, Nsid};
use std::collections::HashMap;
use thiserror::Error;

pub mod atproto;
pub mod dns;

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
}

#[derive(Debug, Eq, PartialEq)]
pub enum Verdict {
    Valid,
    Invalid,
}

#[derive(Debug, Default)]
pub struct _Cache {
    known_valid: HashMap<Nsid, bool>,
}

impl _Cache {
    fn cached_verdict(&self, nsid: &Nsid) -> Option<Verdict> {
        self.known_valid.get(nsid).map(|is_valid| {
            if *is_valid {
                Verdict::Valid
            } else {
                Verdict::Invalid
            }
        })
    }
    fn mark_valid(&mut self, nsid: Nsid) {
        self.known_valid.insert(nsid, true);
    }
    fn mark_invalid(&mut self, nsid: Nsid) {
        self.known_valid.insert(nsid, false);
    }
}

// pub type Cache = std::sync::Arc<std::cell::RefCell<_Cache>>;

#[derive(Debug, Default)]
pub struct Outcome {
    validity: HashMap<Nsid, bool>,
}

impl Outcome {
    fn mark_valid(&mut self, nsid: Nsid) {
        self.validity.insert(nsid, true);
    }
    fn mark_invalid(&mut self, nsid: Nsid) {
        self.validity.insert(nsid, false);
    }
    pub fn ordered_results(&self) -> Vec<(&Nsid, bool)> {
        let mut results = self
            .validity
            .iter()
            .map(|(n, v)| (n, *v))
            .collect::<Vec<(&Nsid, bool)>>();
        results.sort_unstable_by_key(|(n, _)| n.as_str());
        results
    }
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (nsid, is_valid) in self.ordered_results().iter() {
            let s = nsid.as_str();
            let v = if *is_valid { "\u{2705}" } else { "\u{274c}" };
            writeln!(f, "{s}\t{v}")?;
        }
        Ok(())
    }
}

pub async fn check_collection(
    atproto_client: &atproto::Client,
    mut dns_client: dns::Client,
    cache: &mut _Cache,
    nsid: &Nsid,
) -> Result<Verdict, AppError> {
    let address = dns::nsid_address(nsid.as_str());
    if let Ok(did) = dns::get_txt_did(&mut dns_client, address).await {
        let mut cursor: Option<String> = None;
        let mut found = false;
        while let Ok(atproto::FoundLexica {
            lexica,
            cursor: new_cursor,
        }) = atproto::get_lexicon_records(atproto_client, &did, cursor.clone()).await
        {
            for lexicon_nsid in lexica.iter() {
                cache.mark_valid(lexicon_nsid.clone());
                if lexicon_nsid == nsid {
                    found = true;
                }
            }
            if new_cursor.is_none() {
                break;
            } else {
                cursor = new_cursor;
            }
        }
        if found {
            Ok(Verdict::Valid)
        } else {
            cache.mark_invalid(nsid.clone());
            Ok(Verdict::Invalid)
        }
    } else {
        Ok(Verdict::Invalid)
    }
}

pub async fn check_collections(
    cache: &mut _Cache,
    dns_client: &dns::Client,
    atproto_client: &atproto::Client,
    collections: Vec<Nsid>,
) -> Result<Outcome, AppError> {
    let mut outcome = Outcome::default();
    for ns in collections {
        let da = ns.domain_authority();
        if da.starts_with("app.bsky") {
            continue;
        }
        match cache.cached_verdict(&ns) {
            Some(Verdict::Valid) => outcome.mark_valid(ns),
            Some(Verdict::Invalid) => outcome.mark_invalid(ns),
            None => match check_collection(atproto_client, dns_client.clone(), cache, &ns).await? {
                Verdict::Valid => outcome.mark_valid(ns),
                Verdict::Invalid => outcome.mark_invalid(ns),
            },
        }
    }
    Ok(outcome)
}

pub async fn check_user_collections(
    cache: &mut _Cache,
    dns_client: &dns::Client,
    atproto_client: &atproto::Client,
    user_did: &Did,
) -> Result<Outcome, AppError> {
    let user_collections = atproto::get_user_collections(atproto_client, user_did).await?;
    check_collections(cache, dns_client, atproto_client, user_collections).await
}
