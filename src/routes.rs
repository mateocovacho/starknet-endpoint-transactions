use rocket::{get, State};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::config::Config;

#[derive(Serialize, Deserialize)]
struct Transaction {
    from_address: String,
    to_address: String,
}

#[get("/transactions/<address>")]
pub async fn get_related_wallets(address: String, client: &State<Client>, config: &State<Config>) -> String {
    // Fetch first-degree transactions
    let first_degree_url = format!(
        "{}/get_transaction_history?address={}",
        config.rpc_endpoint, address
    );

    let first_degree_txs: Vec<Transaction> = client
        .get(&first_degree_url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let mut wallets: HashSet<String> = HashSet::new();
    wallets.insert(address.clone());

    // Collect first-degree wallets
    for tx in first_degree_txs {
        wallets.insert(tx.from_address);
        wallets.insert(tx.to_address);
    }

    // Fetch second-degree transactions (simplified, assumes same endpoint)
    let mut second_degree_wallets = HashSet::new();
    for wallet in wallets.iter() {
        let second_degree_url = format!(
            "{}/get_transaction_history?address={}",
            config.rpc_endpoint, wallet
        );
        let txs: Vec<Transaction> = client
            .get(&second_degree_url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        for tx in txs {
            second_degree_wallets.insert(tx.from_address);
            second_degree_wallets.insert(tx.to_address);
        }
    }

    // Combine and exclude the original address
    wallets.extend(second_degree_wallets);
    wallets.remove(&address);

    // Return as JSON
    serde_json::to_string(&wallets.into_iter().collect::<Vec<String>>()).unwrap()
}
