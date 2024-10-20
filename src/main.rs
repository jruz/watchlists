use clap::{Parser, Subcommand};

mod exchanges;
mod utils;

use exchanges::{kucoin, binance, woo};

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
}

#[tokio::main]
async fn main() {
    let cli = Cli::try_parse().unwrap_or_else(|e| e.exit());

    match &cli.command {
        Commands::Woo { perp, spot } => {
            if *perp {
                let woo_perp = woo::get_perp().await;
                let woo_perp_name = "WOO-PERP";
                //println!("{woo_perp:#?}\n");
                utils::handle_file(&woo_perp, woo_perp_name);
            }
            if *spot {
                let woo_spot = woo::get_spot().await;
                let woo_spot_name = "WOO-SPOT";
                //println!("{woo_spot:#?}\n");
                utils::handle_file(&woo_spot, woo_spot_name);
            }
        }
        Commands::Binance => {
            let binance_spot = binance::get_spot().await;
            let binance_spot_name = "BINANCE-SPOT";
            //println!("{binance_spot:#?}\n");
            utils::handle_file(&binance_spot, binance_spot_name);
        }
        Commands::Kucoin => {
            let kucoin_spot = kucoin::get_spot().await;
            let kucoin_spot_name = "KUCOIN-SPOT";
            //println!("{kucoin_spot:#?}\n");
            utils::handle_file(&kucoin_spot, kucoin_spot_name);
        }
    }
}
