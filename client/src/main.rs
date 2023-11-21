use std::error::Error;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn handle_client(
    mut client_stream: TcpStream,
    remote_host: &str,
    remote_port: u16,
) -> Result<(), Box<dyn Error>> {
    // Connect to the remote server
    println!("handle connection {:?}", client_stream.peer_addr());
    let mut remote_stream = TcpStream::connect((remote_host, remote_port)).await?;

    // Create a buffer to hold the data
    let mut buffer = [0; 4096];

    loop {
        // Read data from the client
        let num_bytes = client_stream.read(&mut buffer).await?;
        if num_bytes == 0 {
            // Client disconnected
            println!("client disconnet");
            break;
        }
        println!("read from client len {}", num_bytes);

        // Write the length of the received data
        remote_stream
            .write_all(&(num_bytes as u32).to_be_bytes())
            .await?;

        // Write the received data itself
        remote_stream.write_all(&buffer[..num_bytes]).await?;

        // Read the length of data from the remote server
        let mut length_buffer = [0; 4];
        remote_stream.read_exact(&mut length_buffer).await?;

        let length = u32::from_be_bytes(length_buffer);
        let mut data_buffer = vec![0; length as usize];

        // Read the data from the remote server
        remote_stream.read_exact(&mut data_buffer).await?;

        // Write the length and data back to the client
        // client_stream.write_all(&length_buffer).await?;
        client_stream.write_all(&data_buffer).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_host = "127.0.0.1";
    let local_port = 50080;
    let remote_host = "127.0.0.1";
    let remote_port = 28080;

    let local_addr: SocketAddr = format!("{}:{}", local_host, local_port).parse()?;
    let listener = TcpListener::bind(local_addr).await?;

    println!("Server listening on {}:{}", local_host, local_port);

    loop {
        let (client_stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(err) = handle_client(client_stream, remote_host, remote_port).await {
                eprintln!("Error handling client: {}", err);
            }
        });
    }
}
