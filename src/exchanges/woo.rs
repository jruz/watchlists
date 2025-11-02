use reqwest::header;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Response {
    pub rows: Vec<Row>,
    #[allow(dead_code)]
    pub success: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Row {
    pub symbol: String,
    pub is_stable: u32,
    pub is_trading: u32,
}

const EXCHANGE_NAME: &str = "WOONETWORK";

async fn get_data() -> Result<Response, Box<dyn std::error::Error>> {
    let api_url = "https://api.woo.org/v1/public/info";
    let client = reqwest::Client::new();
    let res = client
        .get(api_url)
        .header(
            header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/113.0",
        )
        .send()
        .await?
        .text()
        .await?;

    let parsed: Response = serde_json::from_str(&res)?;

    Ok(parsed)
}

fn filter_symbols(response: Vec<Row>) -> Vec<String> {
    response.iter()
        .filter(|row| row.is_stable == 0 && row.is_trading == 1)
        .map(|row| row.symbol.clone())
        .collect()
}

async fn get_symbols() -> Vec<String> {
    let data = get_data().await;
    match data {
        Ok(data) => filter_symbols(data.rows),
        Err(_) => vec![],
    }
}

pub fn process_perp(symbols: &[String]) -> Vec<String> {
    symbols
        .iter()
        .filter(|symbol| symbol.starts_with("PERP"))
        .filter_map(|symbol| {
            let parts: Vec<&str> = symbol.split('_').collect();
            if parts.len() > 1 {
                Some(parts.get(1..)?.join(""))
            } else {
                None
            }
        })
        .map(|symbol| format!("{EXCHANGE_NAME}:{symbol}.P"))
        .collect()
}

pub fn process_spot(symbols: &[String]) -> Vec<String> {
    symbols
        .iter()
        .filter(|symbol| symbol.starts_with("SPOT"))
        .filter_map(|symbol| {
            let parts: Vec<&str> = symbol.split('_').collect();
            if parts.len() > 1 {
                Some(parts.get(1..)?.join(""))
            } else {
                None
            }
        })
        .map(|symbol| format!("{EXCHANGE_NAME}:{symbol}"))
        .collect()
}

pub async fn get_perp() -> Vec<String> {
    let data = get_symbols().await;
    process_perp(&data)
}

pub async fn get_spot() -> Vec<String> {
    let data = get_symbols().await;
    process_spot(&data)
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_symbols() {
        let rows = vec![
            Row {
                symbol: "PERP_BTC_USDT".to_string(),
                is_stable: 0,
                is_trading: 1,
            },
            Row {
                symbol: "SPOT_ETH_USDT".to_string(),
                is_stable: 0,
                is_trading: 1,
            },
            Row {
                symbol: "SPOT_USDC_USDT".to_string(),
                is_stable: 1,
                is_trading: 1,
            },
            Row {
                symbol: "SPOT_XMR_USDT".to_string(),
                is_stable: 0,
                is_trading: 0,
            },
        ];

        let result = filter_symbols(rows);

        assert_eq!(result.len(), 2);
        assert!(result.contains(&"PERP_BTC_USDT".to_string()));
        assert!(result.contains(&"SPOT_ETH_USDT".to_string()));
    }

    #[test]
    fn test_process_perp() {
        let symbols = vec![
            "PERP_BTC_USDT".to_string(),
            "PERP_ETH_USDT".to_string(),
            "SPOT_XMR_USDT".to_string(),
        ];

        let result = process_perp(&symbols);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "WOONETWORK:BTCUSDT.P");
        assert_eq!(result[1], "WOONETWORK:ETHUSDT.P");
    }

    #[test]
    fn test_process_spot() {
        let symbols = vec![
            "SPOT_BTC_USDT".to_string(),
            "SPOT_ETH_USDT".to_string(),
            "PERP_XMR_USDT".to_string(),
        ];

        let result = process_spot(&symbols);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "WOONETWORK:BTCUSDT");
        assert_eq!(result[1], "WOONETWORK:ETHUSDT");
    }

    #[test]
    fn test_process_perp_output_format() {
        let symbols = vec!["PERP_BTC_USDT".to_string()];

        let result = process_perp(&symbols);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "WOONETWORK:BTCUSDT.P");
        assert!(result[0].contains(':'));
        assert!(result[0].ends_with(".P"));
    }

    #[test]
    fn test_process_spot_output_format() {
        let symbols = vec!["SPOT_ETH_USDT".to_string()];

        let result = process_spot(&symbols);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "WOONETWORK:ETHUSDT");
        assert!(result[0].contains(':'));
    }

    #[test]
    fn test_get_perp_and_spot_from_fixture() {
        let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("woo_response.json");

        if !fixture_path.exists() {
            return;
        }

        let fixture_data = std::fs::read_to_string(fixture_path)
            .expect("Failed to read fixture file");
        let response: Response = serde_json::from_str(&fixture_data)
            .expect("Failed to parse fixture JSON");
        let symbols = filter_symbols(response.rows);

        let perps = process_perp(&symbols);
        let spots = process_spot(&symbols);

        assert!(!perps.is_empty());
        assert!(perps.iter().all(|s| s.starts_with("WOONETWORK:")));
        assert!(perps.iter().all(|s| s.ends_with(".P")));
        assert!(perps.contains(&"WOONETWORK:BTCUSDT.P".to_string()), "WOO should have BTC perp");
        assert!(perps.contains(&"WOONETWORK:ETHUSDT.P".to_string()), "WOO should have ETH perp");

        assert!(!spots.is_empty());
        assert!(spots.iter().all(|s| s.starts_with("WOONETWORK:")));
        assert!(spots.contains(&"WOONETWORK:BTCUSDT".to_string()), "WOO should have BTC spot");
        assert!(spots.contains(&"WOONETWORK:ETHUSDT".to_string()), "WOO should have ETH spot");
    }
}
