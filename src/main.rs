use std::fs::File;
use std::io::{self, BufWriter, Write};

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

#[tokio::main]
async fn main() {
    let perp = woo::get_perp().await;
    let perp_name = "WOO-PERP";
    let spot = woo::get_spot().await;
    let spot_name = "WOO-SPOT";

    //println!("{perp:#?}\n");
    //println!("{spot:#?}\n");

    match write_file(&perp, perp_name) {
        Ok(()) => println!("{perp_name}: {} tickers", &perp.len()),
        Err(e) => println!("Error: {e}"),
    }

    match write_file(&spot, spot_name) {
        Ok(()) => println!("{spot_name}: {} tickers", &spot.len()),
        Err(e) => println!("Error: {e}"),
    }
}
