use anyhow::Result;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::sync::Arc;

use crate::error::AppError;
use crate::models::Transaction;
use crate::rpc::transaction_utils::parse_transaction_from_response;

#[derive(Debug, Clone)]
pub struct RpcClient {
    client: reqwest::Client,
    endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<RpcError>,
    id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

impl RpcClient {
    pub fn new() -> Self {
        let endpoint = env::var("STARKNET_RPC_URL")
            .unwrap_or_else(|_| "https://starknet-mainnet.infura.io/v3/your-api-key".to_string());
        
        Self {
            client: reqwest::Client::new(),
            endpoint,
        }
    }

    pub async fn call<T>(&self, method: &str, params: Value) -> Result<T, AppError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        };

        debug!("RPC Request: {}", serde_json::to_string(&request).unwrap());

        let response = self
            .client
            .post(&self.endpoint)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::RpcError(format!("Failed to send request: {}", e)))?;

        let rpc_response: RpcResponse = response
            .json()
            .await
            .map_err(|e| AppError::RpcError(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(AppError::RpcError(format!(
                "RPC error: {} ({})",
                error.message, error.code
            )));
        }

        let result = rpc_response
            .result
            .ok_or_else(|| AppError::RpcError("No result in response".to_string()))?;

        serde_json::from_value(result)
            .map_err(|e| AppError::RpcError(format!("Failed to deserialize result: {}", e)))
    }

    pub async fn get_blocks_in_range(&self, from_block: u64, to_block: u64) -> Result<Vec<Value>, AppError> {
        let mut blocks = Vec::new();
        
        for block_num in from_block..=to_block {
            let block_id = json!({ "block_number": block_num });
            let block = self.call::<Value>("starknet_getBlockWithTxs", json!([block_id])).await?;
            blocks.push(block);
        }
        
        Ok(blocks)
    }
    
    pub async fn get_latest_block_number(&self) -> Result<u64, AppError> {
        let block_hash_and_number = self.call::<Value>("starknet_blockHashAndNumber", json!([])).await?;
        
        let block_number = block_hash_and_number["block_number"]
            .as_str()
            .ok_or_else(|| AppError::RpcError("Missing block number".to_string()))?;
        
        let block_number = u64::from_str_radix(block_number.trim_start_matches("0x"), 16)
            .map_err(|e| AppError::RpcError(format!("Failed to parse block number: {}", e)))?;
        
        Ok(block_number)
    }

    pub async fn get_transactions_for_address(&self, address: &str) -> Result<Vec<Transaction>, AppError> {
        // Create a filter to get events related to the address
        let events_filter = json!({
            "address": address,
            "from_block": { "block_number": 0 },
            "to_block": "latest",
            "page_size": 50,
            "keys": [],
        });

        let events_response = self.call::<Value>("starknet_getEvents", json!([events_filter])).await?;
        
        let mut transactions = Vec::new();
        
        // Extract transaction hashes from events
        if let Some(events) = events_response["events"].as_array() {
            for event in events {
                if let Some(tx_hash) = event["transaction_hash"].as_str() {
                    // Get the full transaction
                    match self.get_transaction_by_hash(tx_hash).await {
                        Ok(tx) => transactions.push(tx),
                        Err(e) => {
                            error!("Failed to get transaction {}: {}", tx_hash, e);
                            continue;
                        }
                    }
                }
            }
        }
        
        // As an alternative approach, we can also scan recent blocks for transactions
        // This can be useful when the event approach doesn't yield enough results
        if transactions.len() < 10 {
            let latest_block = self.get_latest_block_number().await?;
            let from_block = if latest_block > 1000 { latest_block - 1000 } else { 0 };
            
            let blocks = self.get_blocks_in_range(from_block, latest_block).await?;
            
            for block in blocks {
                if let Some(txs) = block["transactions"].as_array() {
                    for tx in txs {
                        // Check if this transaction involves our target address
                        let tx_from = tx["sender_address"].as_str().unwrap_or("");
                        let tx_to = tx["calldata"].as_array()
                            .and_then(|arr| arr.get(1))
                            .and_then(|val| val.as_str())
                            .unwrap_or("");
                        
                        if tx_from == address || tx_to == address {
                            match parse_transaction_from_response(tx) {
                                Ok(parsed_tx) => transactions.push(parsed_tx),
                                Err(e) => {
                                    error!("Failed to parse transaction: {}", e);
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(transactions)
    }
    
    pub async fn get_transaction_by_hash(&self, tx_hash: &str) -> Result<Transaction, AppError> {
        let transaction = self.call::<Value>("starknet_getTransactionByHash", json!([tx_hash])).await?;
        
        parse_transaction_from_response(&transaction)
    }
}

// Create a module to hold RPC-related functions
pub mod starknet {
    use super::*;
    use crate::models::Transaction;
    use anyhow::Result;
    use std::collections::HashSet;
    
    pub async fn get_transaction_graph(address: &str, depth: usize) -> Result<Vec<Transaction>, AppError> {
        let client = RpcClient::new();
        
        // First degree: get all transactions for the address
        let mut all_transactions = client.get_transactions_for_address(address).await?;
        
        // Get unique addresses that interacted with our target
        let mut related_addresses = all_transactions
            .iter()
            .flat_map(|tx| {
                vec![tx.from_address.clone(), tx.to_address.clone()]
            })
            .filter(|addr| addr != address && addr != "0x0")
            .collect::<HashSet<String>>();
        
        // Second degree: get transactions for related addresses
        if depth > 1 {
            for related_address in related_addresses.iter() {
                let related_txs = client.get_transactions_for_address(related_address).await?;
                all_transactions.extend(related_txs);
            }
        }
        
        Ok(all_transactions)
    }
}
