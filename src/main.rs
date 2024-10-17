use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::{self, BufWriter, Write};

mod binance;
mod woo;

fn write_file(lines: &[String], name: &str) -> io::Result<()> {
    let filename = format!("{}{}{}{}", "dist/", "- C - ", &name, ".txt");
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    for line in lines {
        writeln!(writer, "{line}")?;
    }

    writer.flush()
}

fn handle_file(data: &[String], name: &str) {
    match write_file(data, name) {
        Ok(()) => println!("{name}: {} tickers", &data.len()),
        Err(e) => println!("Error: {e}"),
    }
}

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
                handle_file(&woo_perp, woo_perp_name);
            }
            if *spot {
                let woo_spot = woo::get_spot().await;
                let woo_spot_name = "WOO-SPOT";
                //println!("{woo_spot:#?}\n");
                handle_file(&woo_spot, woo_spot_name);
            }
        }
        Commands::Binance => {
            let binance_spot = binance::get_spot().await;
            let binance_spot_name = "BINANCE-SPOT";
            //println!("{binance_spot:#?}\n");
            handle_file(&binance_spot, binance_spot_name);
        }
    }
}
