use rocket::{routes, Rocket, Build};
use reqwest::Client;
use log::info;
use env_logger;

mod config;
mod routes;

use config::Config;

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
        .mount("/", routes![routes::get_related_wallets]);

    if let Err(e) = rocket.launch().await {
        eprintln!("Rocket launch failed: {}", e);
    }
}
