use std::fs;
use std::path::PathBuf;

fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(filename)
}

async fn fetch_url(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    use reqwest::header;
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header(header::USER_AGENT, "Mozilla/5.0")
        .send()
        .await?
        .text()
        .await?;
    Ok(res)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Generating test fixtures from live APIs...\n");

    eprintln!("→ Fetching Binance data...");
    let res = fetch_url("https://api.binance.com/api/v3/exchangeInfo?permissions=SPOT").await?;
    fs::write(fixture_path("binance_response.json"), &res)?;
    eprintln!("  ✓ Generated binance_response.json");

    eprintln!("→ Fetching KuCoin data...");
    let res = fetch_url("https://api.kucoin.com/api/v1/market/allTickers").await?;
    fs::write(fixture_path("kucoin_response.json"), &res)?;
    eprintln!("  ✓ Generated kucoin_response.json");

    eprintln!("→ Fetching WOO data...");
    let res = fetch_url("https://api.woo.org/v1/public/info").await?;
    fs::write(fixture_path("woo_response.json"), &res)?;
    eprintln!("  ✓ Generated woo_response.json");

    eprintln!("→ Fetching StockAnalysis data (SPY)...");
    let res = fetch_url("https://stockanalysis.com/etf/spy/holdings").await?;
    fs::write(fixture_path("stockanalysis_spy.html"), &res)?;
    eprintln!("  ✓ Generated stockanalysis_spy.html");

    eprintln!("→ Fetching EarningsHub data (this-week) with Playwright...");
    {
        use chrono::{Datelike, Duration, Local};
        use playwright::api::playwright::Playwright;

        let today = Local::now().date_naive();
        let days_since_monday = today.weekday().num_days_from_monday();
        let monday = today
            .checked_sub_signed(Duration::days(i64::from(days_since_monday)))
            .unwrap_or(today);
        let week_date = monday.format("%Y-%m-%d").to_string();

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

        page.wait_for_timeout(10_000.0).await;

        let content = page.content().await?;
        fs::write(fixture_path("earningshub_this_week.html"), &content)?;

        browser.close().await?;
        eprintln!("  ✓ Generated earningshub_this_week.html");
    }

    eprintln!("\n✅ All fixtures generated successfully!");
    eprintln!("   Location: tests/fixtures/");

    Ok(())
}
