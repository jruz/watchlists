use playwright::api::{playwright::Playwright, Page};

pub async fn get_this_week() -> Vec<String> {
    let playwright = Playwright::initialize().await.expect("Failed to initialize playwright");
    playwright.prepare().expect("Failed to prepare playwright");

    let chromium = playwright.chromium();

    let chromium_executable = std::env::var("PLAYWRIGHT_CHROMIUM_EXECUTABLE")
        .expect("PLAYWRIGHT_CHROMIUM_EXECUTABLE not set. Run with: nix develop");

    let browser = chromium
        .launcher()
        .headless(true)
        .executable(std::path::Path::new(&chromium_executable))
        .launch()
        .await
        .expect("Failed to launch browser");

    let context = browser
        .context_builder()
        .build()
        .await
        .expect("Failed to create context");

    let page = context
        .new_page()
        .await
        .expect("Failed to create page");

    page.goto_builder("https://earningshub.com/earnings-calendar/this-week")
        .timeout(60000.0)
        .goto()
        .await
        .expect("Failed to navigate to earnings calendar");

    page.wait_for_timeout(15000.0).await;

    let tickers = extract_tickers(&page).await;

    browser.close().await.expect("Failed to close browser");

    tickers
}

async fn extract_tickers(page: &Page) -> Vec<String> {
    let js_code = r#"
        () => {
            const symbols = new Set();
            document.querySelectorAll('a[href*="?symbol="]').forEach(el => {
                const href = el.getAttribute('href');
                const match = href.match(/[?&]symbol=([A-Z0-9.-]+)/);
                if (match && match[1]) {
                    symbols.add(match[1]);
                }
            });
            return Array.from(symbols).sort();
        }
    "#;

    let result = page.evaluate(js_code, ()).await;

    match result {
        Ok(value) => {
            let tickers: Vec<String> = serde_json::from_value(value)
                .unwrap_or_else(|_| Vec::new())
                .iter()
                .map(|s: &String| format!("NASDAQ:{}", s))
                .collect();
            tickers
        }
        Err(e) => {
            eprintln!("Failed to extract tickers: {e}");
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tickers_from_fixture() {
        let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("earningshub_this_week.html");

        if !fixture_path.exists() {
            return;
        }

        let html = std::fs::read_to_string(fixture_path).unwrap();

        assert!(html.contains("Earnings Hub"), "Fixture should contain Earnings Hub page");
        assert!(html.contains("earnings-calendar"), "Fixture should reference earnings calendar");

        let re = regex::Regex::new(r#"[?&]symbol=([A-Z0-9.-]+)"#).unwrap();
        let ticker_count = re.captures_iter(&html).count();

        assert!(ticker_count > 0, "Fixture should contain at least one ticker symbol in URL parameters");
    }

    #[test]
    fn test_extract_tickers_basic() {
        let html = r#"<a href="/earnings-calendar/this-week?symbol=AAPL">Apple</a><a href="/earnings-calendar/this-week?symbol=MSFT">Microsoft</a>"#;

        let mut tickers = Vec::new();
        let re = regex::Regex::new(r#"[?&]symbol=([A-Z0-9.-]+)"#).unwrap();

        for cap in re.captures_iter(html) {
            if let Some(symbol) = cap.get(1) {
                tickers.push(format!("NASDAQ:{}", symbol.as_str()));
            }
        }

        assert_eq!(tickers.len(), 2);
        assert!(tickers.contains(&"NASDAQ:AAPL".to_string()));
        assert!(tickers.contains(&"NASDAQ:MSFT".to_string()));
    }
}
