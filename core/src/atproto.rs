use crate::AppError;
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

pub type Client = Agent<CredentialSession<MemorySessionStore, ReqwestClient>>;

#[derive(Debug, Clone)]
pub struct FoundLexica {
    pub lexica: Vec<Nsid>,
    pub cursor: Option<String>,
}

pub fn client() -> Client {
    let session = CredentialSession::new(
        ReqwestClient::new("https://bsky.social"),
        MemorySessionStore::default(),
    );
    Agent::new(session)
}

pub async fn get_lexicon_records(
    client: &Client,
    did: &Did,
    cursor: Option<String>,
) -> Result<FoundLexica, AppError> {
    use list_records::*;
    let data = ParametersData {
        cursor,
        collection: Nsid::from_str("com.atproto.lexicon.schema").unwrap(),
        limit: None,
        repo: AtIdentifier::Did(did.clone()),
        reverse: Some(false),
    };
    let params = Parameters {
        data,
        extra_data: ipld_core::ipld::Ipld::Null,
    };
    match client.api.com.atproto.repo.list_records(params).await {
        Err(_) => Err(AppError::AtProtoError),
        Ok(Object {
            data: OutputData { cursor, records },
            ..
        }) => {
            let lexica = records
                .into_iter()
                .map(
                    |Object {
                         data: RecordData { uri, .. },
                         ..
                     }| { Nsid::new(aturi_to_nsid(&uri).to_string()).unwrap() },
                )
                .collect();
            Ok(FoundLexica { cursor, lexica })
        }
    }
}

fn aturi_to_nsid(uri: &str) -> Nsid {
    Nsid::new(uri.rsplit_once('/').unwrap().1.to_string()).unwrap()
}

pub async fn get_user_collections(client: &Client, did: &Did) -> Result<Vec<Nsid>, AppError> {
    use atrium_api::com::atproto::repo::describe_repo::*;
    let input_data = ParametersData {
        repo: AtIdentifier::Did(did.clone()),
    };
    let params = Parameters {
        data: input_data,
        extra_data: ipld_core::ipld::Ipld::Null,
    };
    if let Ok(Object {
        data: OutputData { collections, .. },
        ..
    }) = client.api.com.atproto.repo.describe_repo(params).await
    {
        Ok(collections)
    } else {
        Err(AppError::AtProtoError)
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
}
