use std::net::{IpAddr, SocketAddr};

use local_ip_address::local_ip;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

use crate::svc::Service;

mod svc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let my_local_ip: IpAddr = local_ip().unwrap();
    let addr = SocketAddr::from((my_local_ip, 3000));
    let listener = TcpListener::bind(addr).await?;
    let (tx, _) = broadcast::channel(32);

    let service = Service::new(tx);

    println!(
        "[ SRV ] Server listening on {}",
        listener.local_addr().unwrap()
    );

    loop {
        let (stream, addr) = listener.accept().await?;
        let svc_clone = service.clone();
        tokio::task::spawn(async move {
            svc_clone.call(addr, stream).await;
        });
    }
}
