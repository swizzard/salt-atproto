//! atproto-specific

use crate::SaltError;
use atrium_api::agent::Agent;
use atrium_api::agent::atp_agent::{CredentialSession, store::MemorySessionStore};
use atrium_api::com::atproto::repo::list_records;
use atrium_api::types::{
    Object,
    string::{AtIdentifier, Nsid},
};
use atrium_xrpc_client::reqwest::ReqwestClient;
use std::str::FromStr;

pub use atrium_api::types::string::Did;

pub type AtProtoClient = Agent<CredentialSession<MemorySessionStore, ReqwestClient>>;

/// Nsids retrieved from com.atproto.repo.listRecords
#[derive(Debug, Clone)]
pub struct FoundLexicaNsids {
    pub lexica: Vec<Nsid>,
    pub cursor: Option<String>,
}

/// Create an AtProto client
pub fn atproto_client() -> AtProtoClient {
    let session = CredentialSession::new(
        ReqwestClient::new("https://bsky.social"),
        MemorySessionStore::default(),
    );
    Agent::new(session)
}

#[derive(Debug, Clone)]
struct FoundRecords {
    pub lexica: Vec<list_records::RecordData>,
    pub cursor: Option<String>,
}

async fn get_lexica_records(
    client: &AtProtoClient,
    did: &Did,
    cursor: Option<String>,
) -> Result<FoundRecords, SaltError> {
    use list_records::*;
    let data = ParametersData {
        cursor,
        collection: Nsid::from_str("com.atproto.lexicon.schema").unwrap(),
        limit: None,
        repo: AtIdentifier::Did(did.clone()),
        reverse: Some(false),
    };
    let params = mk_parameters(data);
    match client.api.com.atproto.repo.list_records(params).await {
        Err(_) => Err(SaltError::AtProtoNotFoundError(
            "com.atproto.lexicon.schema".into(),
        )),
        Ok(Object {
            data: OutputData { cursor, records },
            ..
        }) => {
            let lexica = records
                .into_iter()
                .map(|Object { data, .. }| data)
                .collect();
            Ok(FoundRecords { cursor, lexica })
        }
    }
}

/// Retrieve a page of lexicon NSIDs
pub async fn get_lexicon_nsids(
    client: &AtProtoClient,
    did: &Did,
    cursor: Option<String>,
) -> Result<FoundLexicaNsids, SaltError> {
    let FoundRecords { cursor, lexica } = get_lexica_records(client, did, cursor).await?;
    let lexica = lexica
        .into_iter()
        .map(|list_records::RecordData { uri, .. }| {
            Nsid::new(aturi_to_nsid(&uri).to_string()).unwrap()
        })
        .collect();
    Ok(FoundLexicaNsids { lexica, cursor })
}

fn aturi_to_nsid(uri: &str) -> Nsid {
    Nsid::new(uri.rsplit_once('/').unwrap().1.to_string()).unwrap()
}

/// Retrieve all the Nsid of every collection in a repo
pub async fn get_user_collections(
    client: &AtProtoClient,
    did: &Did,
) -> Result<Vec<Nsid>, SaltError> {
    use atrium_api::com::atproto::repo::describe_repo::*;
    let input_data = ParametersData {
        repo: AtIdentifier::Did(did.clone()),
    };
    let params = mk_parameters(input_data);
    if let Ok(Object {
        data: OutputData { collections, .. },
        ..
    }) = client.api.com.atproto.repo.describe_repo(params).await
    {
        Ok(collections)
    } else {
        Err(SaltError::AtProtoNotFoundError("describe_repo".into()))
    }
}

/// Resolve an identity into a Did    
/// CURRENTLY BROKEN
/// REQUIRES LOGGED-IN CLIENT
pub async fn resolve_identity(client: &AtProtoClient, identifier: &str) -> Result<Did, SaltError> {
    use atrium_api::com::atproto::identity::{defs::*, resolve_identity::*};
    let identifier = AtIdentifier::from_str(identifier)
        .map_err(|_| SaltError::IdentifierError(String::from(identifier)))?;
    let input_data = ParametersData { identifier };
    let params = mk_parameters(input_data);
    let resp = client
        .api
        .com
        .atproto
        .identity
        .resolve_identity(params)
        .await;
    if let Ok(Output {
        data: IdentityInfoData { did, .. },
        ..
    }) = resp
    {
        Ok(did)
    } else {
        Err(SaltError::AtProtoError)
    }
}
/// Derive the URI where a NSID's lexicon's DID can be found
pub fn nsid_lexicon_address(nsid: &str) -> String {
    // strip off `name` (last element), reverse remainder
    // e.g. `community.lexicon.calendar.event` -> `_lexicon.calendar.lexicon.community`
    // ill-formedness out of scope
    std::iter::once("_lexicon")
        .chain(nsid.split('.').rev().skip(1))
        .intersperse(".")
        .collect()
}

fn mk_parameters<D>(data: D) -> Object<D> {
    Object {
        data,
        extra_data: ipld_core::ipld::Ipld::Null,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_aturi_to_nsid() {
        let uri = "at://did:plc:zylhqsjug3f76uqxguhviqka/com.atproto.lexicon.schema/blue.2048.verification.stats";
        let expected = Nsid::new("blue.2048.verification.stats".into()).unwrap();
        let actual = aturi_to_nsid(uri);
        assert_eq!(actual, expected)
    }
    #[test]
    fn test_nsid_lexicon_address() {
        let nsid = "community.lexicon.calendar.event";
        let expected = String::from("_lexicon.calendar.lexicon.community");
        let actual = nsid_lexicon_address(nsid);
        assert_eq!(actual, expected)
    }
}
