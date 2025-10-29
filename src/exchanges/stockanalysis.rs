use color_eyre::eyre::Result;
use reqwest::header;
use scraper::{Html, Selector};

async fn get_html(ticker: &str) -> Result<String, serde_json::Error> {
    let api_url = format!("https://stockanalysis.com/etf/{ticker}/holdings");
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

    Ok(res)
}

pub async fn get_components(ticker: &str) -> Result<Vec<String>> {
    let html = get_html(ticker).await?;
    let document = Html::parse_document(&html);
    let table_selector = Selector::parse("#main table tbody").unwrap();
    let ticker_selector = Selector::parse("td a").unwrap();
    let data = document.select(&table_selector).next();
    let ticker_els = data.unwrap().select(&ticker_selector);
    let tickers: Vec<String> = ticker_els.map(|el| el.text().collect::<String>()).collect();

    Ok(tickers)
}
