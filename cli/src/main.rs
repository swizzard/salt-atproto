use clap::{Parser, Subcommand};
use salt_atproto_checker::{Cache, check_user_collections};
use salt_atproto_core::{SaltError, atproto_client, dns_client};
use std::str::FromStr;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Check whether user's collections reference valid lexica
    ValidateLexica { did: String },
}

#[tokio::main]
async fn main() -> Result<(), SaltError> {
    let cli = Cli::parse();
    let atproto_client = atproto_client();
    match &cli.command {
        Commands::ValidateLexica { did } => {
            let dns_client = dns_client().await;
            let mut cache = Cache::default();
            let did = atrium_api::types::string::Did::from_str(did.as_str())
                .map_err(|_| SaltError::DIDError(did.clone()))?;
            let outcome =
                check_user_collections(&mut cache, &dns_client, &atproto_client, &did).await?;
            println!("outcome:\n{outcome}");
        }
    }
    Ok(())
}
