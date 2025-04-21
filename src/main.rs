use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), ()> {
    dotenvy::dotenv().ok();
    let env_map = dotenvy::vars().collect::<std::collections::HashMap<_, _>>();
    let file_name = env_map
        .get("FILE_NAME")
        .expect("FILE_NAME not found in .env");

    let default_out_dir = "./".to_string();
    let out_dir = env_map.get("OUT_DIR").unwrap_or(&default_out_dir);
    let file_path = PathBuf::from(out_dir).join(file_name);

    let default_server_url = "127.0.0.1:9999".to_string();
    let server_url = env_map.get("SERVER_URL").unwrap_or(&default_server_url);
    let mut stream = TcpStream::connect(server_url)
        .await
        .expect("Failed to connect");

    // Read chunk amount
    let mut buffer = [0; 8];
    stream
        .write_all(&file_name.as_bytes())
        .await
        .expect("Failed to send data to server");

    let bytes_read = stream
        .read(&mut buffer)
        .await
        .expect("Failed to read data from server");
    if bytes_read < 8 {
        println!("Err: Server did not send chunk amount");
        return Err(());
    }
    let chunk_amount = u64::from_be_bytes(buffer[..8].try_into().unwrap());

    println!("Total chunks: {}", chunk_amount);

    // Receive anb save file from server
    let mut out_file = File::create(&file_path).expect("Failed to create file");
    let mut chunk = [0; 1024];
    let mut chunks_read: u64 = 0;
    let mut total_bytes_read: u64 = 0;

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
        total_bytes_read += bytes_read as u64;
        out_file
            .write_all(&chunk[..bytes_read])
            .expect("Failed to write data to file");
    }

    println!("File {} downloaded successfully", file_name);
    println!("{} chunks", chunk_amount);
    println!("{} bytes", total_bytes_read);

    return Ok(());
}
