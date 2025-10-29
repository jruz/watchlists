use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

mod exchanges;
mod utils;

use exchanges::{binance, ibkr, kucoin, stockanalysis, woo};

#[derive(Parser)]
#[command(name="Watchlist", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    #[group(required = true, multiple = true)]
    Woo {
        #[arg(long, requires_if("perp", "perp"))]
        perp: bool,
        #[arg(long, requires_if("perp", "perp"))]
        spot: bool,
    },
    Binance,
    Kucoin,
    IBKR,
    StockAnalysis {
        etf: String,
    },
}

fn get_crypto_file_name(name: &str) -> String {
    format!("- C - {name}")
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::try_parse().unwrap_or_else(|e| e.exit());

    match &cli.command {
        Commands::Woo { perp, spot } => {
            if *perp {
                let tickers = woo::get_perp().await;
                let name = get_crypto_file_name("WOO-PERP");

                //println!("{tickers:#?}\n");
                utils::handle_file(&tickers, &name);
            }
            if *spot {
                let tickers = woo::get_spot().await;
                let name = get_crypto_file_name("WOO-SPOT");

                //println!("{tickers:#?}\n");
                utils::handle_file(&tickers, &name);
            }
        }
        Commands::Binance => {
            let tickers = binance::get_spot().await;
            let name = get_crypto_file_name("BINANCE-SPOT");

            //println!("{tickers:#?}\n");
            utils::handle_file(&tickers, &name);
        }
        Commands::Kucoin => {
            let tickers = kucoin::get_spot().await;
            let name = get_crypto_file_name("KUCOIN-SPOT");

            //println!("{tickers:#?}\n");
            utils::handle_file(&tickers, &name);
        }
        Commands::IBKR => {
            let tickers = ibkr::get_tickers().await;

            //println!("{tickers:#?}\n");
            utils::handle_file(&tickers.stocks, "- Positions - Stocks");
            utils::handle_file(&tickers.options, "- Positions - Options");
        }
        Commands::StockAnalysis { etf } => {
            let tickers = stockanalysis::get_components(&etf).await?;

            //println!("{tickers:#?}\n");
            let etf = etf.to_uppercase();
            let file_name = format!("- E - {etf}");
            utils::handle_file(&tickers, &file_name);
        }
    }
    Ok(())
}
