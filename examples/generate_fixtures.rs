use std::fs;
use std::path::PathBuf;

fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(filename)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating test fixtures from live APIs...\n");

    println!("→ Fetching Binance data...");
    {
        use reqwest::header;
        let client = reqwest::Client::new();
        let res = client
            .get("https://api.binance.com/api/v3/exchangeInfo?permissions=SPOT")
            .header(header::USER_AGENT, "Mozilla/5.0")
            .send()
            .await?
            .text()
            .await?;

        fs::write(fixture_path("binance_response.json"), &res)?;
        println!("  ✓ Generated binance_response.json");
    }

    println!("→ Fetching KuCoin data...");
    {
        use reqwest::header;
        let client = reqwest::Client::new();
        let res = client
            .get("https://api.kucoin.com/api/v1/market/allTickers")
            .header(header::USER_AGENT, "Mozilla/5.0")
            .send()
            .await?
            .text()
            .await?;

        fs::write(fixture_path("kucoin_response.json"), &res)?;
        println!("  ✓ Generated kucoin_response.json");
    }

    println!("→ Fetching WOO data...");
    {
        use reqwest::header;
        let client = reqwest::Client::new();
        let res = client
            .get("https://api.woo.org/v1/public/info")
            .header(header::USER_AGENT, "Mozilla/5.0")
            .send()
            .await?
            .text()
            .await?;

        fs::write(fixture_path("woo_response.json"), &res)?;
        println!("  ✓ Generated woo_response.json");
    }

    println!("→ Fetching StockAnalysis data (SPY)...");
    {
        use reqwest::header;
        let client = reqwest::Client::new();
        let res = client
            .get("https://stockanalysis.com/etf/spy/holdings")
            .header(header::USER_AGENT, "Mozilla/5.0")
            .send()
            .await?
            .text()
            .await?;

        fs::write(fixture_path("stockanalysis_spy.html"), &res)?;
        println!("  ✓ Generated stockanalysis_spy.html");
    }

    println!("→ Fetching EarningsHub data (this-week) with Playwright...");
    {
        use playwright::api::playwright::Playwright;
        use chrono::{Datelike, Duration, Local};

        let today = Local::now().date_naive();
        let days_since_monday = today.weekday().num_days_from_monday();
        let monday = today - Duration::days(days_since_monday as i64);
        let week_date = monday.format("%Y-%m-%d").to_string();

        let playwright = Playwright::initialize().await?;
        playwright.prepare()?;

        let chromium = playwright.chromium();

        let chromium_executable = std::env::var("PLAYWRIGHT_CHROMIUM_EXECUTABLE")
            .expect("PLAYWRIGHT_CHROMIUM_EXECUTABLE not set. Run with: nix develop");

        let browser = chromium
            .launcher()
            .headless(true)
            .executable(std::path::Path::new(&chromium_executable))
            .launch()
            .await?;

        let context = browser.context_builder().build().await?;
        let page = context.new_page().await?;

        let url = format!("https://earningshub.com/earnings-calendar/week-of/{}", week_date);
        page.goto_builder(&url)
            .timeout(60000.0)
            .goto()
            .await?;

        page.wait_for_timeout(10000.0).await;

        let content = page.content().await?;
        fs::write(fixture_path("earningshub_this_week.html"), &content)?;

        browser.close().await?;
        println!("  ✓ Generated earningshub_this_week.html");
    }

    println!("\n✅ All fixtures generated successfully!");
    println!("   Location: tests/fixtures/");

    Ok(())
}
