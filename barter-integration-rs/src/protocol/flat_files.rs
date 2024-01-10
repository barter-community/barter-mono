use std::io::prelude::*;
use std::{fs::OpenOptions, io::BufWriter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BacktestMode {
    ToFile,
    FromFile,
    None,
}

// pub async get_file_name() -> String {
//     let rec_time = order_book_l2.received_time;
//     let minute = (rec_time.minute() as f32 / 5.0).floor() as i32 * 5;
//     let formatted = rec_time.format("%Y_%m_%d_%H:").to_string() + minute.to_string().as_str();
//     let file_name = format!("data/binance_l2_{}.dat", formatted);
//     file_name
// }

pub fn write_to_file(data: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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

// pub async fn write_to_file(
//     data: &str,
//     output_path: &str,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     // Open the file in append mode
//     let file = OpenOptions::new()
//         .write(true)
//         .append(true)
//         .create(true)
//         .open(output_path)
//         .await?;

//     // Wrap the file in a buffered writer for efficiency
//     let mut writer = BufWriter::new(file);

//     // Receive messages from the channel and write them to the file
//     writer.write_all(format!("{data:?} \n").as_bytes()).await?;

//     // Flush any remaining output to the file
//     writer.flush().await?;

//     Ok(())
// }
