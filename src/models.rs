use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: u64,
    pub label: String,
    pub group: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeArrow {
    pub enabled: bool,
    pub scale_factor: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeArrows {
    pub to: EdgeArrow,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Edge {
    pub to: u64,
    pub from: u64,
    pub label: String,
    pub gas: f64,
    pub arrows: EdgeArrows,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionGraphResponse {
    pub graph: Graph,
    pub counter: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub from_address: String,
    pub to_address: String,
    pub value: f64,
    pub gas_used: f64,
}
