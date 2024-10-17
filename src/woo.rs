use reqwest::header;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Response {
    pub rows: Vec<Row>,
    #[allow(dead_code)]
    pub success: bool,
}

#[derive(Deserialize, Debug)]
pub struct Row {
    pub symbol: String,
    pub is_stable: u32,
    pub is_trading: u32,
}

const EXCHANGE_NAME: &str = "WOONETWORK";

async fn get_data() -> Result<Response, serde_json::Error> {
    let api_url = "https://api.woo.org/v1/public/info".to_string();
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

    //println!("RAW: {res:#?}");
    let parsed: Response = serde_json::from_str(&res).expect("Failed to parse JSON");

    Ok(parsed)
}

async fn get_symbols() -> Vec<String> {
    let data = get_data().await;
    match data {
        Ok(data) => data
            .rows
            .iter()
            .filter(|row| row.is_stable == 0 && row.is_trading == 1)
            .map(|row| row.symbol.clone())
            .collect(),
        Err(_) => vec![],
    }
}

pub async fn get_perp() -> Vec<String> {
    let data = get_symbols().await;
    data.iter()
        .filter(|symbol| symbol.starts_with("PERP"))
        .map(|symbol| {
            let parts: Vec<&str> = symbol.split('_').collect();
            parts[1..].join("")
        })
        .map(|symbol| EXCHANGE_NAME.to_owned() + ":" + &symbol + ".P")
        .collect()
}

pub async fn get_spot() -> Vec<String> {
    let data = get_symbols().await;
    data.iter()
        .filter(|symbol| symbol.starts_with("SPOT"))
        .map(|symbol| {
            let parts: Vec<&str> = symbol.split('_').collect();
            parts[1..].join("")
        })
        .map(|symbol| EXCHANGE_NAME.to_owned() + ":" + &symbol)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn integration_get_data() {
        let data = get_data().await.unwrap();

        assert!(&data.success);
        assert!(!data.rows.is_empty());
    }

    #[tokio::test]
    async fn integration_get_symbols() {
        let data = get_symbols().await;

        //println!("{data:?}");
        assert!(!data.is_empty());
    }

    #[tokio::test]
    async fn integration_get_perp() {
        let data = get_perp().await;

        //println!("{data:?}");
        assert!(!data.is_empty());
    }

    #[tokio::test]
    async fn integration_get_spot() {
        let data = get_spot().await;

        //println!("{data:?}");
        assert!(!data.is_empty());
    }
}
