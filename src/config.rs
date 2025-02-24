use std::env;
use url::Url;

pub struct Config {
    pub rpc_endpoint: String,
    pub network: Network,
}

#[derive(Debug, PartialEq)]
pub enum Network {
    Mainnet,
    Sepolia,
}

impl Config {
    pub fn new() -> Self {
        // Load environment variables from .env file if present
        dotenv::dotenv().ok();

        // Determine network (default to Sepolia if not set)
        let network_str = env::var("NETWORK")
            .unwrap_or_else(|_| "sepolia".to_string())
            .to_lowercase();
        let network = match network_str.as_str() {
            "mainnet" => Network::Mainnet,
            _ => Network::Sepolia, // Default to Sepolia for any other value
        };

        // Check for custom RPC endpoint
        let custom_rpc = env::var("CUSTOM_RPC_ENDPOINT").ok();
        let rpc_endpoint = match custom_rpc {
            Some(endpoint) => {
                // Validate the custom endpoint
                Url::parse(&endpoint).expect("Invalid CUSTOM_RPC_ENDPOINT URL");
                endpoint
            }
            None => {
                // Use default RPC endpoints based on network
                match network {
                    Network::Mainnet => "https://starknet-mainnet.public.blastapi.io".to_string(),
                    Network::Sepolia => "https://starknet-sepolia.public.blastapi.io".to_string(),
                }
            }
        };

        Config {
            rpc_endpoint,
            network,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::new();
        assert_eq!(config.network, Network::Sepolia);
        assert_eq!(config.rpc_endpoint, "https://starknet-sepolia.public.blastapi.io");
    }
}
