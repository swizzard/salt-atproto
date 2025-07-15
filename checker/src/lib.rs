//! Check NSID validity    
//! For our purposes, an NSID is valid if:    
//!   * there is a DNS TXT record available the NSID's domain authority's `_lexicon` subdomain
//!   * that TXT record contains `did=$LEXICON_REPO_DID`
//!   * `$LEXICON_REPO_DID` contains a `com.atproto.lexicon.schema` record that defines the NSID

use atrium_api::types::string::{Did, Nsid};
use salt_atproto_core::{
    AtProtoClient, DnsClient, SaltError,
    atproto::{FoundLexicaNsids, get_lexicon_nsids, get_user_collections, nsid_lexicon_address},
    dns::get_txt_did,
};
use std::collections::HashMap;

/// Whether a lexicon is valid (retrievable) or invalid
#[derive(Debug, Eq, PartialEq)]
pub enum Verdict {
    Valid,
    Invalid,
}

/// Cache seen NSIDs
#[derive(Debug, Default)]
pub struct Cache {
    known_valid: HashMap<Nsid, bool>,
}

impl Cache {
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

/// Summary of results
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
    /// Results, alphabetical by NSID
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

/// Validate a collection by NSID
pub async fn check_collection(
    atproto_client: &AtProtoClient,
    mut dns_client: DnsClient,
    cache: &mut Cache,
    nsid: &Nsid,
) -> Result<Verdict, SaltError> {
    let address = nsid_lexicon_address(nsid.as_str());
    if let Ok(did) = get_txt_did(&mut dns_client, address).await {
        let mut cursor: Option<String> = None;
        let mut found = false;
        while let Ok(FoundLexicaNsids {
            lexica,
            cursor: new_cursor,
        }) = get_lexicon_nsids(atproto_client, &did, cursor.clone()).await
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

/// Validate multiple collections by NSID    
/// Skips `app.bsky` NSIDs, which are trivially valid
pub async fn check_collections(
    cache: &mut Cache,
    dns_client: &DnsClient,
    atproto_client: &AtProtoClient,
    collections: Vec<Nsid>,
) -> Result<Outcome, SaltError> {
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

/// Retrieve a repo's collections and validate them
pub async fn check_user_collections(
    cache: &mut Cache,
    dns_client: &DnsClient,
    atproto_client: &AtProtoClient,
    user_did: &Did,
) -> Result<Outcome, SaltError> {
    let user_collections = get_user_collections(atproto_client, user_did).await?;
    check_collections(cache, dns_client, atproto_client, user_collections).await
}
