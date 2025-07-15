use salt_atproto_checker::{Cache, check_user_collections};
use salt_atproto_core::{AppError, atproto_client, dns_client};
#[tokio::main]
async fn main() -> Result<(), AppError> {
    let atproto_client = atproto_client();
    let dns_client = dns_client().await;
    let mut cache = Cache::default();
    let did =
        atrium_api::types::string::Did::new("did:plc:xydueznukwpv3esjwcexd676".into()).unwrap();
    let outcome = check_user_collections(&mut cache, &dns_client, &atproto_client, &did).await?;
    println!("outcome:\n{outcome}");
    Ok(())
}
