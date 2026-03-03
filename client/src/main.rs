use std::{io, net::SocketAddr};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc,
};

const PORT: u16 = 3000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[ CLI ] Digite o ip");
    let mut ip = String::new();
    io::stdin().read_line(&mut ip).unwrap();
    let ip = ip.trim().parse::<std::net::IpAddr>().unwrap();

    let addr = SocketAddr::from((ip, PORT));

    let socket = TcpStream::connect(addr).await?;
    let (tx, mut rx) = mpsc::channel::<String>(1);

    tokio::spawn(async move {
        let stdin = io::stdin();
        let mut line = String::new();
        loop {
            line.clear();
            stdin.read_line(&mut line).unwrap();
            if tx.send(line.clone()).await.is_err() {
                break;
            }
        }
    });

    let mut reader = BufReader::new(socket);
    let mut buffer = String::new();

    loop {
        tokio::select! {
            read_result = reader.read_line(&mut buffer) => {
                match read_result {
                    Ok(_bytes_read) => {
                        print!("[ CLI ] {}", buffer);
                        buffer.clear();
                    }
                    Err(e) => {
                        println!("[ CLI ] Error reading stream: {}", e);
                    }
                }
            }

            payload = rx.recv() => {
                match payload {
                    Some(msg) => { reader.write_all(msg.as_bytes()).await?; }
                    None => break,
                }
            }
        }
    }

    Ok(())
}
