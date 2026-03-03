use std::net::SocketAddr;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::broadcast::Sender,
};

#[derive(Debug, Clone)]
pub struct Service {
    publisher: Sender<String>,
}

#[derive(Debug, Clone)]
struct Event(SocketAddr, String);
impl From<Event> for String {
    fn from(event: Event) -> Self {
        format!("{}|{}", event.0, event.1)
    }
}

impl From<String> for Event {
    fn from(value: String) -> Self {
        let mut parts = value.splitn(2, '|');
        let addr = parts.next().unwrap().parse().unwrap();
        let msg = parts.next().unwrap_or("").to_string();
        Event(addr, msg)
    }
}

impl Service {
    pub fn new(publisher: Sender<String>) -> Self {
        Self { publisher }
    }

    pub async fn call(&self, addr: SocketAddr, stream: TcpStream) {
        let mut consumer = self.publisher.subscribe();
        let mut reader = BufReader::new(stream);
        let mut buffer = String::new();
        loop {
            tokio::select! {
                // Recebendo mensagem do cliente
                read_result = reader.read_line(&mut buffer) => {
                    match read_result {
                        Ok(0) => {
                            println!("[ SRV ] Client {} disconnected.", addr);
                            return;
                        }
                        Ok(_) => {
                            self.publisher
                                .send(String::from(Event(addr, buffer.clone())))
                                .expect("Error publishing message.");
                        }
                        Err(e) => {
                            println!("Error reading from {}: {}", addr, e);
                            return;
                        }
                    }
                    buffer.clear();
                }

                // Enviando mensagem para o cliente
                event = consumer.recv() => {
                    match event {
                        Ok(raw) => {
                            let Event(id, message) = Event::from(raw);
                            if id != addr {
                                let formatted_msg = format!("[{}]: {}", id, message);
                                reader.write(formatted_msg.as_bytes()).await.unwrap();
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            println!("Receiver for {} lagged, skipped {} messages.", addr, n);
                        }
                        Err(e) => {
                            println!("Broadcast error for {}: {}", addr, e);
                            return;
                        }
                    }
                }
            }
        }
    }
}
