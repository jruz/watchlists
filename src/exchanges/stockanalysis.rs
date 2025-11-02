use color_eyre::eyre::Result;
use reqwest::header;
use scraper::{Html, Selector};

async fn get_html(ticker: &str) -> Result<String> {
    let api_url = format!("https://stockanalysis.com/etf/{ticker}/holdings");
    let client = reqwest::Client::new();
    let res = client
        .get(&api_url)
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

pub fn parse_html(html: &str) -> Result<Vec<String>> {
    let document = Html::parse_document(html);
    let table_selector = Selector::parse("#main table tbody")
        .map_err(|e| color_eyre::eyre::eyre!("Failed to parse table selector: {:?}", e))?;
    let ticker_selector = Selector::parse("td a")
        .map_err(|e| color_eyre::eyre::eyre!("Failed to parse ticker selector: {:?}", e))?;

    let data = document
        .select(&table_selector)
        .next()
        .ok_or_else(|| color_eyre::eyre::eyre!("Table not found in HTML"))?;

    let ticker_els = data.select(&ticker_selector);
    let tickers: Vec<String> = ticker_els.map(|el| el.text().collect::<String>()).collect();

    Ok(tickers)
}

pub async fn get_components(ticker: &str) -> Result<Vec<String>> {
    let html = get_html(ticker).await?;
    parse_html(&html)
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_html_basic() {
        let html = r#"
            <html>
                <body>
                    <div id="main">
                        <table>
                            <tbody>
                                <tr>
                                    <td><a href="/stocks/aapl/">AAPL</a></td>
                                    <td>Apple Inc.</td>
                                </tr>
                                <tr>
                                    <td><a href="/stocks/msft/">MSFT</a></td>
                                    <td>Microsoft Corporation</td>
                                </tr>
                                <tr>
                                    <td><a href="/stocks/googl/">GOOGL</a></td>
                                    <td>Alphabet Inc.</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </body>
            </html>
        "#;

        let result = parse_html(html).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "AAPL");
        assert_eq!(result[1], "MSFT");
        assert_eq!(result[2], "GOOGL");
    }

    #[test]
    fn test_parse_html_missing_table() {
        let html = r#"
            <html>
                <body>
                    <div id="main">
                        <p>No table here</p>
                    </div>
                </body>
            </html>
        "#;

        let result = parse_html(html);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_html_empty_table() {
        let html = r#"
            <html>
                <body>
                    <div id="main">
                        <table>
                            <tbody>
                            </tbody>
                        </table>
                    </div>
                </body>
            </html>
        "#;

        let result = parse_html(html).unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_get_components_from_fixture() {
        let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("stockanalysis_spy.html");

        if !fixture_path.exists() {
            return;
        }

        let html = std::fs::read_to_string(fixture_path).unwrap();
        let result = parse_html(&html);

        assert!(result.is_ok());
        let tickers = result.unwrap();

        assert!(!tickers.is_empty());
        assert!(tickers.iter().all(|t| !t.is_empty()));

        assert!(
            tickers.contains(&"AAPL".to_string()),
            "SPY should contain AAPL"
        );
        assert!(
            tickers.contains(&"MSFT".to_string()),
            "SPY should contain MSFT"
        );
        assert!(
            tickers.contains(&"NVDA".to_string()),
            "SPY should contain NVDA"
        );
    }
}
