use rocket::serde::json::Json;
use crate::error::AppError;
use crate::models::{Edge, EdgeArrow, EdgeArrows, Graph, Node, TransactionGraphResponse, Transaction};
use crate::rpc::client::starknet;
use log::info;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[get("/api/transactions/<wallet_address>")]
pub async fn get_wallet_transactions(wallet_address: String) -> Result<Json<TransactionGraphResponse>, AppError> {
    info!("Getting transactions for wallet: {}", wallet_address);
    
    // Validate wallet address format
    if !wallet_address.starts_with("0x") || wallet_address.len() != 66 {
        return Err(AppError::InvalidAddress(format!(
            "Invalid StarkNet address format: {}",
            wallet_address
        )));
    }
    
    // Get transactions up to 2nd degree of connection
    let transactions = starknet::get_transaction_graph(&wallet_address, 2).await?;
    
    // Convert transactions to graph representation
    let graph = build_transaction_graph(transactions, &wallet_address);
    
    let response = TransactionGraphResponse {
        graph,
        counter: 76, // This could be dynamic based on some metric
    };
    
    Ok(Json(response))
}

fn build_transaction_graph(transactions: Vec<Transaction>, target_address: &str) -> Graph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();
    let mut node_id_counter: u64 = 106; // Start with 106 as in the example
    
    // Add the target address as the first node
    nodes.insert(
        target_address.to_string(),
        Node {
            id: node_id_counter,
            label: target_address.to_string(),
            group: 1,
        },
    );
    
    node_id_counter += 1;
    
    // Process all transactions
    for tx in transactions {
        // Add nodes for from_address if not exists
        if !nodes.contains_key(&tx.from_address) {
            nodes.insert(
                tx.from_address.clone(),
                Node {
                    id: node_id_counter,
                    label: tx.from_address.clone(),
                    group: 1,
                },
            );
            node_id_counter += 1;
        }
        
        // Add nodes for to_address if not exists
        if !nodes.contains_key(&tx.to_address) {
            nodes.insert(
                tx.to_address.clone(),
                Node {
                    id: node_id_counter,
                    label: tx.to_address.clone(),
                    group: 1,
                },
            );
            node_id_counter += 1;
        }
        
        // Add edge
        let from_id = nodes.get(&tx.from_address).unwrap().id;
        let to_id = nodes.get(&tx.to_address).unwrap().id;
        
        // Format value to ETH
        let value_eth = tx.value / 1e18;
        let label = format!("{:.2} ETH", value_eth);
        
        edges.push(Edge {
            from: from_id,
            to: to_id,
            label,
            gas: tx.gas_used,
            arrows: EdgeArrows {
                to: EdgeArrow {
                    enabled: true,
                    scale_factor: 0.5,
                },
            },
            id: Uuid::new_v4().to_string(),
        });
    }
    
    Graph {
        nodes: nodes.into_values().collect(),
        edges,
    }
}
