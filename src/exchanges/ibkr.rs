use ibapi::accounts::PositionUpdate;
use ibapi::prelude::SecurityType;
use ibapi::Client;

fn get_client() -> Client {
    let connection_url = "127.0.0.1:7496";

    println!("connecting to {}", connection_url);
    let client = Client::connect(connection_url, 100).expect("connection to TWS failed!");
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

    let client = get_client();
    println!("Getting positions");
    let subscription = client.positions().expect("error requesting positions");
    for position_response in subscription.iter() {
        match position_response {
            PositionUpdate::Position(position) => match position.contract.security_type {
                SecurityType::Stock => {
                    //println!("{:#?}", position);
                    if position.position == 0.0 {
                        continue;
                    }
                    let ticker = position.contract.symbol.to_string();
                    let exchange = match position.contract.exchange.as_str() {
                        "SMART" => "NYSE",
                        "ISLAND" => "NASDAQ",
                        "PINK" => "OTC",
                        "IBIS" => "XETR",
                        "IBIS2" => "XETR",
                        "BVME" => "MIL",
                        "SBF" => "EURONEXT",
                        _ => &position.contract.exchange,
                    };
                    stocks.push(format!("{exchange}:{ticker}"));
                }
                SecurityType::Option => {
                    if position.position == 0.0 {
                        continue;
                    }
                    //println!("{:#?}", position);
                    let ticker = position.contract.symbol.to_string();
                    options.push(ticker);
                }
                _ => continue,
            },
            PositionUpdate::PositionEnd => break,
        }
    }

    Tickers { stocks, options }
}
