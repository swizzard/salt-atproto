use std::net::SocketAddr;
use std::str::FromStr;

use crate::AppError;
use hickory_client::client::{Client, ClientHandle};
use hickory_client::proto::rr::{DNSClass, Name, RData, RecordType, rdata::txt::TXT};
use hickory_client::proto::runtime::TokioRuntimeProvider;
use hickory_client::proto::udp::UdpClientStream;

pub async fn dns_client() -> Client {
    let address = SocketAddr::from(([8, 8, 8, 8], 53));
    let conn = UdpClientStream::builder(address, TokioRuntimeProvider::default()).build();
    let (client, bg) = Client::connect(conn).await.unwrap();
    tokio::spawn(bg);
    client
}

pub async fn get_txt_did(client: &mut Client, address: String) -> Result<String, AppError> {
    let resp = client
        .query(
            Name::from_str(address.as_ref()).unwrap(),
            DNSClass::IN,
            RecordType::TXT,
        )
        .await?;
    if let RData::TXT(txt) = resp
        .answers()
        .first()
        .ok_or(AppError::MissingTXTError(address.clone()))?
        .data()
    {
        txt_did(txt)
    } else {
        Err(AppError::MissingTXTError(address))
    }
}

pub fn nsid_address(nsid: String) -> String {
    // strip off `name` (last element), reverse remainder
    // e.g. `community.lexicon.calendar.event` -> `_lexicon.calendar.lexicon.community`
    // ill-formedness out of scope
    std::iter::once("_lexicon")
        .chain(nsid.split('.').rev().skip(1))
        .intersperse(".")
        .collect()
}

fn txt_did(txt: &TXT) -> Result<String, AppError> {
    let s = format!("{txt}");
    s.strip_prefix("did=")
        .ok_or(AppError::DIDError(s.clone()))
        .map(String::from)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_nsid_address() {
        let nsid = String::from("community.lexicon.calendar.event");
        let expected = String::from("_lexicon.calendar.lexicon.community");
        let actual = nsid_address(nsid);
        assert_eq!(actual, expected)
    }
}
