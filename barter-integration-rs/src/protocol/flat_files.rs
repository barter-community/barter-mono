use std::io::prelude::*;
use std::{fs::OpenOptions, io::BufWriter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BacktestMode {
    ToFile,
    FromFile,
    None,
}

pub fn append_to_file(data: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Open the file in append mode
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(output_path)?;

    // Wrap the file in a buffered writer for efficiency
    let mut writer = BufWriter::new(&file);

    // Receive messages from the channel and write them to the file
    writer.write_all(format!("{data}\n").as_bytes())?;

    // Flush any remaining output to the file
    writer.flush()?;

    Ok(())
}
