use crate::constants::{SOLFI_MARKETS, USDC, WSOL};
use crate::types::{AccountWithAddress, FetchMetadata};
use solana_pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use spl_associated_token_account::get_associated_token_address;

pub async fn fetch_and_persist_accounts(rpc_url: String) -> eyre::Result<()> {
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    let addresses: Vec<Pubkey> = [WSOL, USDC]
        .into_iter()
        .chain(SOLFI_MARKETS.iter().flat_map(|market| {
            [
                *market,
                get_associated_token_address(market, &WSOL),
                get_associated_token_address(market, &USDC),
            ]
        }))
        .collect();
    tracing::info!("Fetching accounts");
    let slot_lower = client.get_slot().await?;
    let accounts = client.get_multiple_accounts(&addresses).await?;
    let slot_upper = client.get_slot().await?;
    let results = accounts
        .iter()
        .zip(addresses)
        .filter_map(|(account, address)| {
            Some(AccountWithAddress {
                address,
                account: account.clone()?,
            })
        })
        .collect::<Vec<_>>();

    for result in &results {
        result.save_to_file()?;
    }

    let metadata = FetchMetadata {
        slot_lower,
        slot_upper,
    };
    metadata.save_to_file()?;
    tracing::info!("Done");

    Ok(())
}
