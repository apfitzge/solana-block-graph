use {
    clap::Parser,
    graphia_input::{GraphiaInput, GraphiaInputEdge, GraphiaInputNode, GraphiaInputNodeMetaData},
    prio_graph::{AccessKind, PrioGraph, TopLevelId},
    solana_client::{rpc_client::RpcClient, rpc_config::RpcBlockConfig},
    solana_sdk::{clock::Slot, pubkey::Pubkey},
    solana_transaction_status::{TransactionDetails, UiLoadedAddresses, UiTransactionEncoding},
    std::{path::PathBuf, str::FromStr},
};

mod graphia_input;

#[derive(Parser, Debug)]
struct Cli {
    /// Slot to get the block for
    #[clap(long, short)]
    slot: Slot,
    /// Output file path
    #[clap(long, short)]
    output: PathBuf,
    /// Show all connected edges, not just final blocking edges.
    #[clap(long, short)]
    verbose: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Id(usize);

impl TopLevelId<Id> for Id {
    fn id(&self) -> Id {
        *self
    }
}

fn main() {
    let Cli {
        slot,
        output,
        verbose,
    } = Cli::parse();
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or("https://api.mainnet-beta.solana.com".to_string());

    let rpc_client = RpcClient::new(rpc_url);
    let block = rpc_client
        .get_block_with_config(
            slot,
            RpcBlockConfig {
                encoding: Some(UiTransactionEncoding::Base64),
                transaction_details: Some(TransactionDetails::Full),
                rewards: None,
                commitment: None,
                max_supported_transaction_version: Some(0),
            },
        )
        .expect("failed to fetch block");

    let transactions = block.transactions.expect("block must have transactions");

    let mut prio_graph = PrioGraph::new(|id, _| *id);
    let mut decoded_transactions = vec![];
    for (transaction_index, transaction) in transactions.iter().enumerate() {
        let meta = transaction
            .meta
            .as_ref()
            .expect("transactions must have meta");
        let transaction = transaction
            .transaction
            .clone()
            .decode()
            .expect("failed to decode transaction");

        let loaded_addresses = Option::<UiLoadedAddresses>::from(meta.loaded_addresses.clone())
            .expect("transaction must have loaded addresses");

        let mut write_accounts = vec![];
        let mut read_accounts = vec![];

        // Add static keys into read/write vecs
        for (index, key) in transaction.message.static_account_keys().iter().enumerate() {
            if transaction.message.is_maybe_writable(index) {
                write_accounts.push(*key);
            } else {
                read_accounts.push(*key);
            }
        }

        // Add dynamic keys into read/write vecs
        write_accounts.extend(
            loaded_addresses
                .writable
                .iter()
                .map(|key| Pubkey::from_str(&key).expect("failed to parse key")),
        );
        read_accounts.extend(
            loaded_addresses
                .readonly
                .iter()
                .map(|key| Pubkey::from_str(&key).expect("failed to parse key")),
        );

        let fee = meta.fee;
        let cus = Option::<u64>::from(meta.compute_units_consumed.clone()).unwrap_or(0);
        decoded_transactions.push((transaction, fee, cus));

        // Skip any votes. (TODO: Make this check more robust)
        if read_accounts.contains(&solana_sdk::vote::program::ID) {
            continue;
        }
        // Insert the transaction into the prio-graph
        prio_graph.insert_transaction(
            Id(transaction_index),
            write_accounts
                .into_iter()
                .map(|key| (key, AccessKind::Write))
                .chain(read_accounts.into_iter().map(|key| (key, AccessKind::Read))),
        );
    }

    // Now pop from the graph to construct graphia input (json)
    let mut graphia_input = GraphiaInput::default();
    let mut edge_count: u32 = 0;
    let mut depth: usize = 0;
    while !prio_graph.is_empty() {
        let mut popped = Vec::new();
        depth += 1;

        while let Some(id) = prio_graph.pop() {
            popped.push(id);

            // Insert a new node into the graphia input graph.
            let (tx, fee, cus) = &decoded_transactions[id.0];
            graphia_input.graph.nodes.push(GraphiaInputNode {
                id: id.0.to_string(),
                metadata: GraphiaInputNodeMetaData {
                    signature: tx.signatures[0].to_string(),
                    num_signatures: tx.signatures.len(),
                    fee: *fee,
                    compute: *cus,
                    depth,
                },
            });
        }

        for popped in popped {
            let unblocked = prio_graph.unblock(&popped);

            // Add edges to graphia input graph.
            for target in unblocked {
                if !prio_graph.is_blocked(target) || verbose {
                    graphia_input.graph.edges.push(GraphiaInputEdge {
                        id: edge_count.to_string(),
                        source: popped.0.to_string(),
                        target: target.0.to_string(),
                    });
                }
                edge_count += 1;
            }
        }
    }

    let file = std::fs::File::options()
        .write(true)
        .create(true)
        .append(false)
        .truncate(true)
        .open(output)
        .expect("failed to write file");
    serde_json::to_writer(file, &graphia_input).unwrap();
}
