use std::net::SocketAddr;
use std::str::FromStr;

use crate::AppError;
use crate::atproto::Did;
use hickory_client::client::ClientHandle;
use hickory_client::proto::rr::{DNSClass, Name, RData, RecordType, rdata::txt::TXT};
use hickory_client::proto::runtime::TokioRuntimeProvider;
use hickory_client::proto::udp::UdpClientStream;

pub use hickory_client::client::Client as DnsClient;

pub async fn dns_client() -> DnsClient {
    let address = SocketAddr::from(([8, 8, 8, 8], 53));
    let conn = UdpClientStream::builder(address, TokioRuntimeProvider::default()).build();
    let (client, bg) = DnsClient::connect(conn).await.unwrap();
    tokio::spawn(bg);
    client
}

pub async fn get_txt_did(client: &mut DnsClient, address: String) -> Result<Did, AppError> {
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

fn txt_did(txt: &TXT) -> Result<Did, AppError> {
    Did::from_str(format!("{txt}").trim_start_matches("did="))
        .map_err(|s| AppError::DIDError(s.into()))
}
