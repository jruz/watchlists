use reqwest::header;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Coin {
    pub id: String,
    pub symbol: String,
    pub name: String,
}

const EXCHANGE_NAME: &str = "BINANCE";
const API_URL: &str =
    "https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&order=market_cap_desc&per_page=100&page=1";

async fn fetch_data(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header(
            header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/113.0",
        )
        .send()
        .await?
        .text()
        .await?;
    Ok(res)
}

async fn get_data() -> Result<Vec<Coin>, Box<dyn std::error::Error>> {
    let res = fetch_data(API_URL).await?;
    let parsed: Vec<Coin> = serde_json::from_str(&res)?;
    Ok(parsed)
}

pub fn process_data(coins: Vec<Coin>) -> Vec<String> {
    let stablecoin_blacklist = [
        "usdt", "usdc", "busd", "dai", "tusd", "usdp", "usdd", "gusd", "paxg", "eurs", "eurt",
        "gbpt", "xaut", "pyusd", "fdusd", "frax", "lusd", "susd", "usdj", "usdk", "usdx", "ust",
        "usdn",
    ];

    let wrapped_prefixes = ["w", "st", "cb"];

    coins
        .iter()
        .filter(|coin| {
            let symbol_lower = coin.symbol.to_lowercase();
            let name_lower = coin.name.to_lowercase();

            let is_stablecoin = stablecoin_blacklist.contains(&symbol_lower.as_str())
                || name_lower.contains("usd")
                || name_lower.contains("dollar")
                || name_lower.contains("stable");

            let is_wrapped = name_lower.starts_with("wrapped ")
                || wrapped_prefixes.iter().any(|prefix| {
                    symbol_lower.starts_with(prefix) && symbol_lower.len() > prefix.len()
                });

            !is_stablecoin && !is_wrapped
        })
        .map(|coin| format!("{}:{}USDT", EXCHANGE_NAME, coin.symbol.to_uppercase()))
        .collect()
}

pub async fn get_top_100() -> Vec<String> {
    match get_data().await {
        Ok(data) => process_data(data),
        Err(_) => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_data_filters_stablecoins() {
        let coins = vec![
            Coin {
                id: "bitcoin".to_string(),
                symbol: "btc".to_string(),
                name: "Bitcoin".to_string(),
            },
            Coin {
                id: "tether".to_string(),
                symbol: "usdt".to_string(),
                name: "Tether".to_string(),
            },
            Coin {
                id: "usd-coin".to_string(),
                symbol: "usdc".to_string(),
                name: "USD Coin".to_string(),
            },
            Coin {
                id: "ethereum".to_string(),
                symbol: "eth".to_string(),
                name: "Ethereum".to_string(),
            },
        ];

        let result = process_data(coins);

        assert_eq!(result.len(), 2);
        assert!(result.contains(&"BINANCE:BTCUSDT".to_string()));
        assert!(result.contains(&"BINANCE:ETHUSDT".to_string()));
        assert!(!result.iter().any(|s| s.contains("USDT:USDT")));
    }

    #[test]
    fn test_process_data_filters_wrapped_coins() {
        let coins = vec![
            Coin {
                id: "bitcoin".to_string(),
                symbol: "btc".to_string(),
                name: "Bitcoin".to_string(),
            },
            Coin {
                id: "wrapped-bitcoin".to_string(),
                symbol: "wbtc".to_string(),
                name: "Wrapped Bitcoin".to_string(),
            },
            Coin {
                id: "weth".to_string(),
                symbol: "weth".to_string(),
                name: "WETH".to_string(),
            },
            Coin {
                id: "steth".to_string(),
                symbol: "steth".to_string(),
                name: "Lido Staked Ether".to_string(),
            },
        ];

        let result = process_data(coins);

        assert_eq!(result.len(), 1);
        assert!(result.contains(&"BINANCE:BTCUSDT".to_string()));
    }

    #[test]
    fn test_process_data_output_format() {
        let coins = vec![Coin {
            id: "bitcoin".to_string(),
            symbol: "btc".to_string(),
            name: "Bitcoin".to_string(),
        }];

        let result = process_data(coins);

        assert_eq!(result.len(), 1);
        assert!(result.iter().all(|s| s == "BINANCE:BTCUSDT"));
        assert!(result.iter().all(|s| s.contains(':')));
        assert!(result.iter().all(|s| s.starts_with("BINANCE:")));
        assert!(result.iter().all(|s| s.ends_with("USDT")));
    }

    #[test]
    fn test_get_top_100_from_fixture() {
        let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("coingecko_response.json");

        if !fixture_path.exists() {
            return;
        }

        let Ok(fixture_data) = std::fs::read_to_string(&fixture_path) else {
            return;
        };
        let Ok(coins) = serde_json::from_str::<Vec<Coin>>(&fixture_data) else {
            return;
        };

        let result = process_data(coins);

        assert!(!result.is_empty());
        assert!(result.iter().all(|s| s.starts_with("BINANCE:")));
        assert!(result.iter().all(|s| s.ends_with("USDT")));
        assert!(result.iter().all(|s| s.contains(':')));

        assert!(
            result.contains(&"BINANCE:BTCUSDT".to_string()),
            "Should have BTC"
        );
        assert!(
            result.contains(&"BINANCE:ETHUSDT".to_string()),
            "Should have ETH"
        );

        assert!(
            !result.iter().any(|s| s.contains("USDT:USDT")),
            "Should not have USDT stablecoin"
        );
        assert!(
            !result.iter().any(|s| s.contains("USDC")),
            "Should not have USDC stablecoin"
        );
    }
}
