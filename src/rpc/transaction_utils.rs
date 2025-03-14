use crate::error::AppError;
use crate::models::Transaction;
use serde_json::Value;

pub fn parse_transaction_from_response(tx_data: &Value) -> Result<Transaction, AppError> {
    // Extract transaction hash
    let hash = tx_data["transaction_hash"]
        .as_str()
        .ok_or_else(|| AppError::RpcError("Missing transaction hash".to_string()))?
        .to_string();
    
    // Extract sender address
    let from_address = match tx_data["type"].as_str() {
        Some("INVOKE") => tx_data["sender_address"]
            .as_str()
            .ok_or_else(|| AppError::RpcError("Missing sender address".to_string()))?,
        Some("DECLARE") => tx_data["sender_address"]
            .as_str()
            .ok_or_else(|| AppError::RpcError("Missing sender address".to_string()))?,
        Some("DEPLOY") => tx_data["contract_address"]
            .as_str()
            .ok_or_else(|| AppError::RpcError("Missing contract address".to_string()))?,
        Some(tx_type) => return Err(AppError::RpcError(format!("Unsupported transaction type: {}", tx_type))),
        None => return Err(AppError::RpcError("Missing transaction type".to_string())),
    }
    .to_string();
    
    // Extract recipient address - this is more complex and depends on the transaction type
    // For invoke transactions, we need to look at calldata
    let to_address = match tx_data["type"].as_str() {
        Some("INVOKE") => {
            if let Some(calldata) = tx_data["calldata"].as_array() {
                if calldata.len() > 1 {
                    calldata[1].as_str().unwrap_or("0x0").to_string()
                } else {
                    "0x0".to_string()
                }
            } else {
                "0x0".to_string()
            }
        },
        Some("DEPLOY") => "0x0".to_string(), // No recipient for deploy
        Some("DECLARE") => "0x0".to_string(), // No recipient for declare
        _ => "0x0".to_string(),
    };
    
    // Extract value - this is complex, as it depends on the transaction data structure
    // In StarkNet, value is typically in the calldata for INVOKE transactions
    let value = match tx_data["type"].as_str() {
        Some("INVOKE") => {
            if let Some(calldata) = tx_data["calldata"].as_array() {
                if calldata.len() > 2 {
                    // Try to parse the value from calldata
                    // This is a simplification - in a real app, you would need to
                    // understand the specific contract and method being called
                    let value_str = calldata[2].as_str().unwrap_or("0x0");
                    u64::from_str_radix(value_str.trim_start_matches("0x"), 16)
                        .unwrap_or(0) as f64 / 1e18
                } else {
                    0.0
                }
            } else {
                0.0
            }
        },
        _ => 0.0,
    };
    
    // Extract gas - we can use the max_fee field as an approximation
    let gas_used = match tx_data["max_fee"].as_str() {
        Some(fee_str) => {
            let fee = u64::from_str_radix(fee_str.trim_start_matches("0x"), 16).unwrap_or(0);
            fee as f64 / 1e18
        },
        None => 2.1e-14, // Default value from the example
    };
    
    Ok(Transaction {
        hash,
        from_address,
        to_address,
        value,
        gas_used,
    })
}
