use reqwest::header;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ResponseTicker {
    pub symbol: String,
}

#[derive(Deserialize, Debug)]
pub struct ResponseData {
    pub ticker: Vec<ResponseTicker>,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    pub data: ResponseData,
}

async fn get_data() -> Result<Response, serde_json::Error> {
    let api_url = "https://api.kucoin.com/api/v1/market/allTickers".to_string();
    let client = reqwest::Client::new();
    let res = client
        .get(api_url)
        .header(
            header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/113.0",
        )
        .send()
        .await
        .expect("Failed to get data")
        .text()
        .await
        .expect("Failed to get body");

    //println!("{res:#?}\n");
    let parsed: Response = serde_json::from_str(&res).expect("Failed to parse JSON");

    Ok(parsed)
}

pub fn filter_data(data: Vec<ResponseTicker>) -> Vec<String> {
    let regex = regex::Regex::new(r"3L|3S|2L|2S|DOWN").unwrap();

    data.iter()
        .map(|row| {
            let parts: Vec<&str> = row.symbol.split('-').collect();
            let base = parts[0];
            let quote = parts[1];
            (base, quote)
        })
        .filter(|(base, quote)| *quote == "USDT" && !regex.is_match(base) && !base.ends_with("UP") && !base.ends_with("DOWN"))
        .map(|(base, _)| base.to_string())
        .collect()
}

pub async fn get_spot() -> Vec<String> {
    if let Ok(data) = get_data().await {
        let tickers = filter_data(data.data.ticker);
        return tickers
            .iter()
            .map(|ticker| format!("KUCOIN:{ticker}USDT"))
            .collect();
    } else {
        println!("Failed to get data");
    }
    vec![]
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_filter_data_nonusdt() {
        let data: Vec<ResponseTicker> = vec![
            ResponseTicker {
                symbol: "BTC-USDT".to_string(),
            },
            ResponseTicker {
                symbol: "ETH-USDT".to_string(),
            },
            ResponseTicker {
                symbol: "XMR-ETH".to_string(),
            },
        ];
        let result = filter_data(data);
        let expected = vec!["BTC".to_string(), "ETH".to_string()];

        assert_eq!(result, expected)
    }

    #[tokio::test]
    async fn test_filter_data_levered() {
        let data: Vec<ResponseTicker> = vec![
            ResponseTicker {
                symbol: "BTC-USDT".to_string(),
            },
            ResponseTicker {
                symbol: "ETH-USDT".to_string(),
            },
            ResponseTicker {
                symbol: "BTC3L-USDT".to_string(),
            },
            ResponseTicker {
                symbol: "BTC3S-USDT".to_string(),
            },
            ResponseTicker {
                symbol: "BTC2L-USDT".to_string(),
            },
            ResponseTicker {
                symbol: "BTC2S-USDT".to_string(),
            },
            ResponseTicker {
                symbol: "WLDUP-USDT".to_string(),
            },
            ResponseTicker {
                symbol: "SUPER-USDT".to_string(),
            },
            ResponseTicker {
                symbol: "JUP-USDT".to_string(),
            },
            ResponseTicker {
                symbol: "BTCDOWN-USDT".to_string(),
            },
        ];
        let result = filter_data(data);
        let expected = vec![
          "BTC".to_string(),
          "ETH".to_string(),
          "SUPER".to_string(),
        ];

        assert_eq!(result, expected)
    }
}
