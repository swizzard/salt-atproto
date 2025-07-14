use lexicon_pruner::{AppError, dns};
#[tokio::main]
async fn main() -> Result<(), AppError> {
    let mut client = dns::dns_client().await;
    let nsid = String::from("community.lexicon.calendar.event");
    let address = dns::nsid_address(nsid);
    let did = dns::get_txt_did(&mut client, address).await?;
    println!("got did: {did}");
    Ok(())
}
