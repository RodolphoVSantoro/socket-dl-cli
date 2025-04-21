use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let env_map = dotenvy::vars().collect::<std::collections::HashMap<_, _>>();
    let file_name = env_map
        .get("FILE_NAME")
        .expect("FILE_NAME not found in .env");
    let mut stream = TcpStream::connect("127.0.0.1:9999")
        .await
        .expect("Failed to connect");
    stream
        .write_all(&file_name.as_bytes())
        .await
        .expect("Failed to send data to server");
    let mut out_file = File::create(&file_name).expect("Failed to create file");
    let mut chunk = [0; 1024];

    let bytes_read = stream
        .read(&mut chunk)
        .await
        .expect("Failed to read data from server");
    if bytes_read == 0 {
        return Ok(());
    }

    let chunk_amount = u64::from_be_bytes(chunk[..8].try_into().unwrap());
    let mut chunks_read: u64 = 0;
    println!("Total chunks: {}", chunk_amount);

    loop {
        let percent_update_limit = 30;
        let is_one_of_every_nth =
            chunks_read % (chunk_amount as f64 / percent_update_limit as f64).ceil() as u64 == 0;
        if chunk_amount < percent_update_limit || is_one_of_every_nth {
            println!("{}%", chunks_read as f64 / chunk_amount as f64 * 100.0);
        }
        chunks_read += 1;

        let bytes_read = stream
            .read(&mut chunk)
            .await
            .expect("Failed to read data from server");
        if bytes_read == 0 {
            break;
        }
        out_file
            .write_all(&chunk[..bytes_read])
            .expect("Failed to write data to file");
    }
    out_file.flush().expect("Failed to flush file");

    println!("File {} downloaded successfully", file_name);
    println!("Total chunks: {}", chunk_amount);
    return Ok(());
}
