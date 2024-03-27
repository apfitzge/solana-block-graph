use {
    clap::Parser,
    solana_client::{rpc_client::RpcClient, rpc_config::RpcBlockConfig},
    solana_sdk::clock::Slot,
    solana_transaction_status::{TransactionDetails, UiTransactionEncoding},
};

#[derive(Parser, Debug)]
struct Cli {
    /// Slot to get the block for
    #[clap(long, short)]
    slot: Slot,
}

fn main() {
    let Cli { slot } = Cli::parse();
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or("https://api.mainnet-beta.solana.com".to_string());

    let rpc_client = RpcClient::new(rpc_url);
    let block = rpc_client
        .get_block_with_config(
            slot,
            RpcBlockConfig {
                encoding: Some(UiTransactionEncoding::Base64),
                transaction_details: Some(TransactionDetails::Accounts),
                rewards: None,
                commitment: None,
                max_supported_transaction_version: Some(0),
            },
        )
        .expect("failed to fetch block");

    println!("{:#?}", block);
}
