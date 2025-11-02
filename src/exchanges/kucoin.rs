use reqwest::header;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponseTicker {
    pub symbol: String,
    #[serde(deserialize_with = "vol_deserializer")]
    #[serde(serialize_with = "vol_serializer")]
    pub vol: f64,
}

fn vol_deserializer<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    let f = s.parse::<f64>().map_err(serde::de::Error::custom)?;
    Ok(f)
}

fn vol_serializer<S>(vol: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&vol.to_string())
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponseData {
    pub ticker: Vec<ResponseTicker>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Response {
    pub data: ResponseData,
}

async fn get_data() -> Result<Response, Box<dyn std::error::Error>> {
    let api_url = "https://api.kucoin.com/api/v1/market/allTickers";
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

pub fn process_data(mut tickers: Vec<ResponseTicker>) -> Vec<String> {
    let Ok(regex) = regex::Regex::new(r"3L|3S|2L|2S|DOWN") else {
        return Vec::new();
    };

    tickers.sort_by(|a, b| b.vol.partial_cmp(&a.vol).unwrap_or(std::cmp::Ordering::Equal));

    tickers.iter()
        .filter_map(|row| {
            let parts: Vec<&str> = row.symbol.split('-').collect();
            let base = parts.get(0)?;
            let quote = parts.get(1)?;
            Some(((*base).to_string(), (*quote).to_string()))
        })
        .filter(|(base, quote)| quote == "USDT" && !regex.is_match(base) && !base.ends_with("UP") && !base.ends_with("DOWN"))
        .map(|(base, _)| base)
        .collect()
}

fn get_spot_impl(response: Response) -> Vec<String> {
    let tickers = process_data(response.data.ticker);
    tickers
        .iter()
        .map(|ticker| format!("KUCOIN:{ticker}USDT"))
        .collect()
}

pub async fn get_spot() -> Vec<String> {
    match get_data().await {
        Ok(data) => get_spot_impl(data),
        Err(e) => {
            eprintln!("Failed to get data: {e}");
            vec![]
        }
    }
}


#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_process_data_sorted() {
        let data: Vec<ResponseTicker> = vec![
            ResponseTicker {
                symbol: "BTC-USDT".to_string(),
                vol: 100_000.0,
            },
            ResponseTicker {
                symbol: "ETH-USDT".to_string(),
                vol: 100.0,
            },
            ResponseTicker {
                symbol: "XMR-USDT".to_string(),
                vol: 100_000_000.0,
            },
        ];
        let result = process_data(data);
        let expected = vec!["XMR".to_string(), "BTC".to_string(), "ETH".to_string()];

        assert_eq!(result, expected)
    }

    #[test]
    fn test_process_data_nonusdt() {
        let data: Vec<ResponseTicker> = vec![
            ResponseTicker {
                symbol: "BTC-USDT".to_string(),
                vol: 100.0,
            },
            ResponseTicker {
                symbol: "ETH-USDT".to_string(),
                vol: 100.0,
            },
            ResponseTicker {
                symbol: "XMR-ETH".to_string(),
                vol: 100.0,
            },
        ];
        let result = process_data(data);
        let expected = vec!["BTC".to_string(), "ETH".to_string()];

        assert_eq!(result, expected)
    }

    #[test]
    fn test_process_data_levered() {
        let data: Vec<ResponseTicker> = vec![
            ResponseTicker {
                symbol: "BTC-USDT".to_string(),
                vol: 100.0,
            },
            ResponseTicker {
                symbol: "ETH-USDT".to_string(),
                vol: 100.0,
            },
            ResponseTicker {
                symbol: "BTC3L-USDT".to_string(),
                vol: 100.0,
            },
            ResponseTicker {
                symbol: "BTC3S-USDT".to_string(),
                vol: 100.0,
            },
            ResponseTicker {
                symbol: "BTC2L-USDT".to_string(),
                vol: 100.0,
            },
            ResponseTicker {
                symbol: "BTC2S-USDT".to_string(),
                vol: 100.0,
            },
            ResponseTicker {
                symbol: "WLDUP-USDT".to_string(),
                vol: 100.0,
            },
            ResponseTicker {
                symbol: "SUPER-USDT".to_string(),
                vol: 100.0,
            },
            ResponseTicker {
                symbol: "JUP-USDT".to_string(),
                vol: 100.0,
            },
            ResponseTicker {
                symbol: "BTCDOWN-USDT".to_string(),
                vol: 100.0,
            },
        ];
        let result = process_data(data);
        let expected = vec![
          "BTC".to_string(),
          "ETH".to_string(),
          "SUPER".to_string(),
        ];

        assert_eq!(result, expected)
    }

    #[test]
    fn test_process_data_output_format() {
        let data: Vec<ResponseTicker> = vec![ResponseTicker {
            symbol: "BTC-USDT".to_string(),
            vol: 100.0,
        }];

        let result = process_data(data);

        assert_eq!(result, vec!["BTC".to_string()]);
    }

    #[test]
    fn test_get_spot_from_fixture() {
        let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("kucoin_response.json");

        if !fixture_path.exists() {
            return;
        }

        let fixture_data = std::fs::read_to_string(fixture_path)
            .expect("Failed to read fixture file");
        let response: Response = serde_json::from_str(&fixture_data)
            .expect("Failed to parse fixture JSON");
        let tickers: Vec<String> = get_spot_impl(response);

        assert!(!tickers.is_empty());
        assert!(tickers.iter().all(|s| s.starts_with("KUCOIN:")));
        assert!(tickers.iter().all(|s| s.contains("USDT")));

        assert!(tickers.contains(&"KUCOIN:BTCUSDT".to_string()), "KuCoin should have BTC");
        assert!(tickers.contains(&"KUCOIN:ETHUSDT".to_string()), "KuCoin should have ETH");
    }

}
