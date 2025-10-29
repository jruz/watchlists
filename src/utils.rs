use std::fs::File;
use std::io::{self, BufWriter, Write};

pub fn write_file(lines: &[String], name: &str) -> io::Result<()> {
    let filename = format!("dist/{name}.txt");
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    for line in lines {
        writeln!(writer, "{line}")?;
    }

    writer.flush().into()
}

pub fn handle_file(data: &[String], name: &str) {
    match write_file(data, name) {
        Ok(()) => println!("{name}: {} tickers", &data.len()),
        Err(e) => println!("Error: {e:#?}"),
    }
}
