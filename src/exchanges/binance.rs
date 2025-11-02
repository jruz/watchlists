use reqwest::header;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Response {
    pub symbols: Vec<Symbol>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Symbol {
    pub symbol: String,
    pub status: String,
    #[serde(rename = "quoteAsset")]
    pub quote_asset: String,
    #[serde(rename = "baseAsset")]
    pub base_asset: String,
}

const EXCHANGE_NAME: &str = "BINANCE";
const API_URL: &str = "https://api.binance.com/api/v3/exchangeInfo?permissions=SPOT";

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

async fn get_data() -> Result<Response, Box<dyn std::error::Error>> {
    let res = fetch_data(API_URL).await?;
    let parsed: Response = serde_json::from_str(&res)?;
    Ok(parsed)
}

pub fn process_data(response: Response) -> Vec<String> {
    let blacklist = [
        "TUSD", "USDC", "BUSD", "EUR", "GBP", "PAX", "DAI", "AUD", "USDP", "FDUSD", "WBTC",
    ];

    response
        .symbols
        .iter()
        .filter(|row| {
            row.status == "TRADING"
                && row.quote_asset == "USDT"
                && !blacklist.contains(&row.base_asset.as_str())
        })
        .map(|row| format!("{EXCHANGE_NAME}:{}", row.symbol))
        .collect()
}

pub async fn get_spot() -> Vec<String> {
    match get_data().await {
        Ok(data) => process_data(data),
        Err(_) => vec![],
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_process_data_filters_usdt_pairs() {
        let response = Response {
            symbols: vec![
                Symbol {
                    symbol: "BTCUSDT".to_string(),
                    status: "TRADING".to_string(),
                    quote_asset: "USDT".to_string(),
                    base_asset: "BTC".to_string(),
                },
                Symbol {
                    symbol: "ETHUSDT".to_string(),
                    status: "TRADING".to_string(),
                    quote_asset: "USDT".to_string(),
                    base_asset: "ETH".to_string(),
                },
                Symbol {
                    symbol: "BTCBUSD".to_string(),
                    status: "TRADING".to_string(),
                    quote_asset: "BUSD".to_string(),
                    base_asset: "BTC".to_string(),
                },
            ],
        };

        let result = process_data(response);

        assert_eq!(result.len(), 2);
        assert!(result.contains(&"BINANCE:BTCUSDT".to_string()));
        assert!(result.contains(&"BINANCE:ETHUSDT".to_string()));
        assert!(!result.iter().any(|s| s.contains("BUSD")));
    }

    #[test]
    fn test_process_data_filters_non_trading() {
        let response = Response {
            symbols: vec![
                Symbol {
                    symbol: "BTCUSDT".to_string(),
                    status: "TRADING".to_string(),
                    quote_asset: "USDT".to_string(),
                    base_asset: "BTC".to_string(),
                },
                Symbol {
                    symbol: "ETHUSDT".to_string(),
                    status: "HALT".to_string(),
                    quote_asset: "USDT".to_string(),
                    base_asset: "ETH".to_string(),
                },
            ],
        };

        let result = process_data(response);

        assert_eq!(result.len(), 1);
        assert!(result.contains(&"BINANCE:BTCUSDT".to_string()));
    }

    #[test]
    fn test_process_data_filters_blacklisted() {
        let response = Response {
            symbols: vec![
                Symbol {
                    symbol: "BTCUSDT".to_string(),
                    status: "TRADING".to_string(),
                    quote_asset: "USDT".to_string(),
                    base_asset: "BTC".to_string(),
                },
                Symbol {
                    symbol: "USDCUSDT".to_string(),
                    status: "TRADING".to_string(),
                    quote_asset: "USDT".to_string(),
                    base_asset: "USDC".to_string(),
                },
                Symbol {
                    symbol: "BUSDUSDT".to_string(),
                    status: "TRADING".to_string(),
                    quote_asset: "USDT".to_string(),
                    base_asset: "BUSD".to_string(),
                },
                Symbol {
                    symbol: "WBTCUSDT".to_string(),
                    status: "TRADING".to_string(),
                    quote_asset: "USDT".to_string(),
                    base_asset: "WBTC".to_string(),
                },
            ],
        };

        let result = process_data(response);

        assert_eq!(result.len(), 1);
        assert!(result.contains(&"BINANCE:BTCUSDT".to_string()));
    }

    #[test]
    fn test_process_data_output_format() {
        let response = Response {
            symbols: vec![Symbol {
                symbol: "BTCUSDT".to_string(),
                status: "TRADING".to_string(),
                quote_asset: "USDT".to_string(),
                base_asset: "BTC".to_string(),
            }],
        };

        let result = process_data(response);

        assert_eq!(result.len(), 1);
        assert!(result.iter().all(|s| s == "BINANCE:BTCUSDT"));
        assert!(result.iter().all(|s| s.contains(':')));
    }

    #[test]
    fn test_get_spot_from_fixture() {
        let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("binance_response.json");

        if !fixture_path.exists() {
            eprintln!("Skipping test: fixture file not found");
            return;
        }

        let fixture_data = std::fs::read_to_string(&fixture_path)
            .expect("Failed to read binance fixture file - file may be corrupted");
        let response: Response = serde_json::from_str(&fixture_data)
            .expect("Failed to parse binance fixture JSON - file may be corrupted");

        let result = process_data(response);

        assert!(!result.is_empty());
        assert!(result.iter().all(|s| s.starts_with("BINANCE:")));
        assert!(result.iter().all(|s| s.contains("USDT")));
        assert!(result.iter().all(|s| s.contains(':')));

        assert!(
            result.contains(&"BINANCE:BTCUSDT".to_string()),
            "Binance should have BTC"
        );
        assert!(
            result.contains(&"BINANCE:ETHUSDT".to_string()),
            "Binance should have ETH"
        );
    }
}
