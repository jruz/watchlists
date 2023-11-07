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

#[tokio::main]
async fn main() {
    let woo_perp = woo::get_perp().await;
    let woo_perp_name = "WOO-PERP";
    let woo_spot = woo::get_spot().await;
    let woo_spot_name = "WOO-SPOT";
    let binance_spot = binance::get_spot().await;
    let binance_spot_name = "BINANCE-SPOT";

    //println!("{woo_perp:#?}\n");
    //println!("{woo_spot:#?}\n");
    //println!("{binance_spot:#?}\n");

    handle_file(&woo_perp, woo_perp_name);
    handle_file(&woo_spot, woo_spot_name);
    handle_file(&binance_spot, binance_spot_name);
}
