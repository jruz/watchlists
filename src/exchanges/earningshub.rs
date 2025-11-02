use playwright::api::{playwright::Playwright, Page};

pub async fn get_earnings_week(week_date: &str) -> Vec<String> {
    match get_earnings_week_impl(week_date).await {
        Ok(tickers) => tickers,
        Err(e) => {
            eprintln!("Failed to fetch earnings data: {e}");
            Vec::new()
        }
    }
}

async fn get_earnings_week_impl(
    week_date: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let playwright = Playwright::initialize().await?;
    playwright.prepare()?;

    let chromium = playwright.chromium();

    let chromium_executable = std::env::var("PLAYWRIGHT_CHROMIUM_EXECUTABLE")
        .map_err(|_| "PLAYWRIGHT_CHROMIUM_EXECUTABLE not set. Run with: nix develop")?;

    let browser = chromium
        .launcher()
        .headless(true)
        .executable(std::path::Path::new(&chromium_executable))
        .launch()
        .await?;

    let context = browser.context_builder().build().await?;

    let page = context.new_page().await?;

    let url = format!("https://earningshub.com/earnings-calendar/week-of/{week_date}");
    page.goto_builder(&url).timeout(60_000.0).goto().await?;

    page.wait_for_timeout(15_000.0).await;

    let tickers = extract_tickers(&page).await;

    browser.close().await?;

    Ok(tickers)
}

async fn extract_tickers(page: &Page) -> Vec<String> {
    let js_code = r#"
        () => {
            const symbols = [];
            const seen = new Set();
            document.querySelectorAll('a[href*="?symbol="]').forEach(el => {
                const href = el.getAttribute('href');
                const match = href.match(/[?&]symbol=([A-Z0-9.-]+)/);
                if (match && match[1] && !seen.has(match[1])) {
                    symbols.push(match[1]);
                    seen.add(match[1]);
                }
            });
            return symbols;
        }
    "#;

    let result = page.evaluate(js_code, ()).await;

    match result {
        Ok(value) => serde_json::from_value(value).unwrap_or_else(|_| Vec::new()),
        Err(e) => {
            eprintln!("Failed to extract tickers: {e}");
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_tickers_from_fixture() {
        let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("earningshub_this_week.html");

        if !fixture_path.exists() {
            return;
        }

        let Ok(html) = std::fs::read_to_string(fixture_path) else {
            return;
        };

        assert!(
            html.contains("Earnings Hub"),
            "Fixture should contain Earnings Hub page"
        );
        assert!(
            html.contains("earnings-calendar"),
            "Fixture should reference earnings calendar"
        );

        let Ok(re) = regex::Regex::new(r"[?&]symbol=([A-Z0-9.-]+)") else {
            return;
        };
        let ticker_count = re.captures_iter(&html).count();

        assert!(
            ticker_count > 0,
            "Fixture should contain at least one ticker symbol in URL parameters"
        );
    }

    #[test]
    fn test_extract_tickers_preserves_document_order() {
        let html = r#"
            <a href="/earnings-calendar/week-of/2025-11-10?symbol=TSLA">Tesla</a>
            <a href="/earnings-calendar/week-of/2025-11-10?symbol=AAPL">Apple</a>
            <a href="/earnings-calendar/week-of/2025-11-10?symbol=MSFT">Microsoft</a>
            <a href="/earnings-calendar/week-of/2025-11-10?symbol=GOOG">Google</a>
        "#;

        let mut tickers = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let Ok(re) = regex::Regex::new(r"[?&]symbol=([A-Z0-9.-]+)") else {
            return;
        };

        for cap in re.captures_iter(html) {
            if let Some(symbol) = cap.get(1) {
                let sym = symbol.as_str();
                if !seen.contains(sym) {
                    tickers.push(sym.to_string());
                    seen.insert(sym.to_string());
                }
            }
        }

        assert_eq!(tickers.len(), 4);
        let expected = vec!["TSLA", "AAPL", "MSFT", "GOOG"];
        assert!(tickers.iter().zip(&expected).all(|(a, b)| a == b));
    }

    #[test]
    fn test_extract_tickers_deduplicates() {
        let html = r#"
            <a href="/earnings-calendar/week-of/2025-11-10?symbol=AAPL">Apple Morning</a>
            <a href="/earnings-calendar/week-of/2025-11-10?symbol=MSFT">Microsoft</a>
            <a href="/earnings-calendar/week-of/2025-11-10?symbol=AAPL">Apple Afternoon</a>
            <a href="/earnings-calendar/week-of/2025-11-10?symbol=TSLA">Tesla</a>
        "#;

        let mut tickers = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let Ok(re) = regex::Regex::new(r"[?&]symbol=([A-Z0-9.-]+)") else {
            return;
        };

        for cap in re.captures_iter(html) {
            if let Some(symbol) = cap.get(1) {
                let sym = symbol.as_str();
                if !seen.contains(sym) {
                    tickers.push(sym.to_string());
                    seen.insert(sym.to_string());
                }
            }
        }

        assert_eq!(tickers.len(), 3);
        let expected = vec!["AAPL", "MSFT", "TSLA"];
        assert!(tickers.iter().zip(&expected).all(|(a, b)| a == b));
    }
}
