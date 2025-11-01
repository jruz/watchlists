use ibapi::accounts::PositionUpdate;
use ibapi::prelude::SecurityType;
use ibapi::Client;

async fn get_client() -> Client {
    let connection_url = "127.0.0.1:7496";

    println!("connecting to {}", connection_url);
    let client = Client::connect(connection_url, 100).await.expect("connection to TWS failed!");
    println!("connected successfully");
    client
}

pub struct Tickers {
    pub stocks: Vec<String>,
    pub options: Vec<String>,
}

pub async fn get_tickers() -> Tickers {
    let mut stocks = Vec::new();
    let mut options = Vec::new();

    let client = get_client().await;
    println!("Getting positions");
    let mut subscription = client.positions().await.expect("error requesting positions");

    while let Some(position_result) = subscription.next().await {
        match position_result {
            Ok(PositionUpdate::Position(position)) => match position.contract.security_type {
                SecurityType::Stock => {
                    if position.position == 0.0 {
                        continue;
                    }
                    let ticker = position.contract.symbol.to_string();
                    let exchange = match position.contract.exchange.to_string().as_str() {
                        "SMART" => "NYSE",
                        "ISLAND" => "NASDAQ",
                        "PINK" => "OTC",
                        "IBIS" => "XETR",
                        "IBIS2" => "XETR",
                        "BVME" => "MIL",
                        "SBF" => "EURONEXT",
                        _ => &position.contract.exchange.to_string(),
                    };
                    stocks.push(format!("{exchange}:{ticker}"));
                }
                SecurityType::Option => {
                    if position.position == 0.0 {
                        continue;
                    }
                    let ticker = position.contract.symbol.to_string();
                    options.push(ticker);
                }
                _ => continue,
            },
            Ok(PositionUpdate::PositionEnd) => break,
            Err(e) => {
                eprintln!("Error receiving position: {e}");
                break;
            }
        }
    }

    Tickers { stocks, options }
}
