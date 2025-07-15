// did:plc:zylhqsjug3f76uqxguhviqka
use salt_atproto::{AppError, atproto, dns};
#[tokio::main]
async fn main() -> Result<(), AppError> {
    let atproto_client = atproto::client();
    let dns_client = dns::dns_client().await;
    let mut cache = salt_atproto::_Cache::default();
    let did =
        atrium_api::types::string::Did::new("did:plc:xydueznukwpv3esjwcexd676".into()).unwrap();
    let outcome =
        salt_atproto::check_user_collections(&mut cache, &dns_client, &atproto_client, &did)
            .await?;
    println!("outcome:\n{outcome}");
    // let mut client = dns::dns_client().await;
    // let nsid = "blue.2048.player.profile";
    // let address = dns::nsid_address(nsid);
    // let did = dns::get_txt_did(&mut client, address).await?;
    // println!("got did: {did:?}");
    Ok(())
}
