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

    println!("\n✅ All fixtures generated successfully!");
    println!("   Location: tests/fixtures/");

    Ok(())
}
