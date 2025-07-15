// did:plc:zylhqsjug3f76uqxguhviqka
use salt_atproto_core::{AppError, atproto, dns};
#[tokio::main]
async fn main() -> Result<(), AppError> {
    let atproto_client = atproto::client();
    let dns_client = dns::dns_client().await;
    let mut cache = salt_atproto_core::_Cache::default();
    let did =
        atrium_api::types::string::Did::new("did:plc:xydueznukwpv3esjwcexd676".into()).unwrap();
    let outcome =
        salt_atproto_core::check_user_collections(&mut cache, &dns_client, &atproto_client, &did)
            .await?;
    println!("outcome:\n{outcome}");
    Ok(())
}
