use crate::message::{ClientMessage, ServerMessage};
use log::{error, info, warn};
use prost::Message;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 512];
        let mut message_data = Vec::new();

        loop {
            let bytes_read = self.stream.read(&mut buffer)?;
            if bytes_read == 0 {
                info!("Client disconnected.");
                return Ok(());
            }
            message_data.extend_from_slice(&buffer[..bytes_read]);

            if let Ok(client_message) = ClientMessage::decode(&message_data[..]) {
                match client_message.message {
                    Some(crate::message::client_message::Message::EchoMessage(echo_message)) => {
                        info!("Received Echo: {}", echo_message.content);
                        let server_message = ServerMessage {
                            message: Some(crate::message::server_message::Message::EchoMessage(echo_message)),
                        };
                        let payload = server_message.encode_to_vec();
                        self.stream.write_all(&payload)?;
                        self.stream.flush()?;
                    }
                    Some(crate::message::client_message::Message::AddRequest(add_request)) => {
                        info!("Received Add Request: {} + {}", add_request.a, add_request.b);
                        let result = add_request.a + add_request.b;
                        let server_message = ServerMessage {
                            message: Some(crate::message::server_message::Message::AddResponse(crate::message::AddResponse { result })),
                        };
                        let payload = server_message.encode_to_vec();
                        self.stream.write_all(&payload)?;
                        self.stream.flush()?;
                    }
                    None => error!("Received empty client message."),
                }
                message_data.clear();
            }
        }
    }
}

pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
}

impl Server {
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let is_running = Arc::new(AtomicBool::new(false));
        Ok(Server {
            listener,
            is_running,
        })
    }

    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst);
        info!("Server is running on {}", self.listener.local_addr()?);

        self.listener.set_nonblocking(true)?;

        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);
                    stream.set_nonblocking(true)?;
                    let is_running = self.is_running.clone(); // Clone for the thread
                    thread::spawn(move || {
                        let mut client = Client::new(stream);
                        while is_running.load(Ordering::SeqCst) { // Check the flag within the client handling loop as well
                            match client.handle() {
                                Ok(()) => (),
                                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                    thread::sleep(Duration::from_millis(10));
                                    continue;
                                }
                                Err(e) => {
                                    error!("Client handling error: {}", e);
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                    if e.kind() != ErrorKind::Interrupted {
                        return Err(e);
                    }
                    break;
                }
            }
        }

        info!("Server stopped.");
        Ok(())
    }

    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}