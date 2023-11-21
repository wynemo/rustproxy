use std::error::Error;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time;

async fn forward_data(
    mut previous_stream: TcpStream,
    remote_host: &str,
    remote_port: u16,
) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 4096];
    let mut final_stream = TcpStream::connect((remote_host, remote_port)).await?;

    loop {
        // Read data length from the previous server
        let mut length_buffer = [0; 4];
        previous_stream.read_exact(&mut length_buffer).await?;

        let length = u32::from_be_bytes(length_buffer);

        // Read data from the previous server
        let mut data_buffer = vec![0; length as usize];
        previous_stream.read_exact(&mut data_buffer).await?;

        // Write data to the final server
        final_stream.write_all(&data_buffer).await?;

        // Read response from the final server with a timeout
        if let Ok(Ok(num_bytes)) =
            time::timeout(Duration::from_secs(2), final_stream.read(&mut buffer)).await
        {
            if num_bytes == 0 {
                break Ok(());
            }
            previous_stream.write_all(&buffer[..num_bytes]).await?;
        } else {
            println!("timeout");
            continue;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_host = "127.0.0.1";
    let local_port = 28080;
    let final_host = "10.0.1.248";
    let final_port = 50080;

    let local_addr: SocketAddr = format!("{}:{}", local_host, local_port).parse()?;
    let listener = TcpListener::bind(local_addr).await?;

    println!("Server listening on {}:{}", local_host, local_port);

    loop {
        let (client_stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(err) = forward_data(client_stream, final_host, final_port).await {
                eprintln!("Error handling client: {}", err);
            }
        });
    }
}
