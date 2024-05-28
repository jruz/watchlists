use reqwest::header;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Response {
    pub symbols: Vec<Symbol>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Symbol {
    pub symbol: String,
    pub status: String,
    #[serde(rename = "quoteAsset")]
    pub quote_asset: String,
    #[serde(rename = "baseAsset")]
    pub base_asset: String,
}

const EXCHANGE_NAME: &str = "BINANCE";

async fn get_data() -> Result<Response, serde_json::Error> {
    let api_url = "https://api.binance.com/api/v3/exchangeInfo?permissions=SPOT".to_string();
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

pub async fn get_spot() -> Vec<String> {
    let blacklist = [
        "TUSD", "USDC", "BUSD", "EUR", "GBP", "PAX", "DAI", "AUD", "USDP", "FDUSD",
    ];
    let data = get_data().await;
    match data {
        Ok(data) => data
            .symbols
            .iter()
            .filter(|row| {
                row.status == "TRADING"
                    && row.quote_asset == "USDT"
                    && !blacklist.contains(&row.base_asset.as_str())
            })
            .map(|row| row.symbol.clone())
            .map(|symbol| EXCHANGE_NAME.to_owned() + ":" + &symbol)
            .collect(),
        Err(_) => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn integration_get_data() {
        let data = get_data().await.unwrap();

        assert!(!data.symbols.is_empty());
    }

    #[tokio::test]
    async fn integration_get_symbols() {
        let data = get_spot().await;

        //println!("{data:?}");
        assert!(!data.is_empty());
    }
}
