use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

mod exchanges;
mod utils;

use exchanges::{binance, earningshub, ibkr, kucoin, stockanalysis, woo};

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
    #[allow(clippy::upper_case_acronyms)]
    IBKR,
    Components {
        etf: String,
    },
    #[command(subcommand)]
    Earnings(EarningsCommands),
}

#[derive(Subcommand)]
enum EarningsCommands {
    ThisWeek,
    NextWeek,
    TwoWeeks,
}

fn get_crypto_file_name(name: &str) -> String {
    format!("- C - {name}")
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

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
        Commands::Components { etf } => {
            let tickers = stockanalysis::get_components(etf).await?;

            //println!("{tickers:#?}\n");
            let etf = etf.to_uppercase();
            let file_name = format!("- E - {etf}");
            utils::handle_file(&tickers, &file_name);
        }
        Commands::Earnings(earnings_cmd) => {
            use chrono::{Datelike, Duration, Local};

            let today = Local::now().date_naive();

            let monday = match earnings_cmd {
                EarningsCommands::ThisWeek => {
                    let days_since_monday = today.weekday().num_days_from_monday();
                    today
                        .checked_sub_signed(Duration::days(i64::from(days_since_monday)))
                        .unwrap_or(today)
                }
                EarningsCommands::NextWeek => {
                    let days_since_monday = today.weekday().num_days_from_monday();
                    let this_monday = today
                        .checked_sub_signed(Duration::days(i64::from(days_since_monday)))
                        .unwrap_or(today);
                    this_monday
                        .checked_add_signed(Duration::days(7))
                        .unwrap_or(this_monday)
                }
                EarningsCommands::TwoWeeks => {
                    let days_since_monday = today.weekday().num_days_from_monday();
                    let this_monday = today
                        .checked_sub_signed(Duration::days(i64::from(days_since_monday)))
                        .unwrap_or(today);
                    this_monday
                        .checked_add_signed(Duration::days(14))
                        .unwrap_or(this_monday)
                }
            };

            let week_date = monday.format("%Y-%m-%d").to_string();
            let tickers = earningshub::get_earnings_week(&week_date).await;

            let file_name = match earnings_cmd {
                EarningsCommands::ThisWeek => "- Earnings - This Week",
                EarningsCommands::NextWeek => "- Earnings - Next Week",
                EarningsCommands::TwoWeeks => "- Earnings - Two Weeks",
            };

            utils::handle_file(&tickers, file_name);
        }
    }
    Ok(())
}
