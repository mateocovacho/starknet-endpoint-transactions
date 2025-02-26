use rocket::{routes, Rocket, Build, serde::json::Json, State};
use reqwest::Client;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

mod config;
mod routes;

use config::Config;
use routes::{ get_related_wallets, AppState};

#[derive(Debug, Deserialize)]
struct WalletRequest {
    wallet_address: String,
}

#[derive(Debug, Serialize)]
struct TransactionGraph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(Debug, Serialize, Clone)]
struct Node {
    id: u64, 
    label: String,
    group: u8,
}

#[derive(Debug, Serialize)]
struct Edge {
    from: u64,
    to: u64,
    label: String,
    gas: f64,
    id: String,
}

struct AppState {
    rpc_client: reqwest::Client,
    rpc_url: String,
}

#[post("/tx-graph", data = "<wallet>")]
async fn get_transaction_graph(
    wallet: Json<WalletRequest>,
    state: &State<AppState>,
) -> Json<TransactionGraph> {
    let mut graph = TransactionGraph {
        nodes: vec![],
        edges: vec![],
    };

    // Fetch initial transactions using block methods
    let (mut nodes, edges) = fetch_transaction_data(&wallet.wallet_address, &state.rpc_client, &state.rpc_url).await;
    graph.nodes.append(&mut nodes);
    graph.edges.extend(edges);

    // Second degree fetch
    let mut second_degree_nodes = graph.nodes.clone();
    for node in second_degree_nodes.drain(..) {
        let (mut new_nodes, new_edges) = fetch_transaction_data(&node.label, &state.rpc_client, &state.rpc_url).await;
        graph.nodes.append(&mut new_nodes);
        graph.edges.extend(new_edges);
    }

    Json(graph)
}

async fn fetch_transaction_data(
    address: &str,
    client: &reqwest::Client,
    rpc_url: &str,
) -> (Vec<Node>, Vec<Edge>) {
    // 1. Get recent blocks using starknet_blockNumber/starknet_getBlockWithTxHashes
    // 2. For each block, get transactions using starknet_getBlockWithTxs
    // 3. Filter transactions related to target address
    // 4. Process transaction details using starknet_getTransactionReceipt
    
    // Transaction data parsing implementation here...
    
    (vec![], vec![]) // Return empty stub for example
}

#[rocket::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Load configuration
    let config = Config::new();
    info!("Using RPC endpoint: {}", config.rpc_endpoint);
    info!("Network: {:?}", config.network);

    // Create HTTP client
    let client = Client::new();

    // Build and launch Rocket
    let rocket: Rocket<Build> = rocket::build()
        .manage(client)
        .manage(config)
        .manage(AppState {
            rpc_client: client,
            rpc_url: config.rpc_endpoint,
        })
        .mount("/", routes![get_transaction_graph, get_related_wallets]);

    if let Err(e) = rocket.launch().await {
        eprintln!("Rocket launch failed: {}", e);
    }
}
