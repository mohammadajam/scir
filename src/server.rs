use std::collections::HashMap;
use std::io::{Read, Write, Result, stdin};
use std::net::{TcpStream, TcpListener, SocketAddr};
use std::thread;

const PORT: u32 = 4000;

struct Client {
    connection: TcpStream,

}

impl Client {
    fn new(stream: TcpStream) -> Self {
        Self {
            connection: stream,
        }
    }
}

struct Server {
    clients: HashMap<SocketAddr, Client>
}

impl Server {
    fn new() -> Self {
        Self { clients: HashMap::new() }
    }

    fn insert_client(&mut self, socket: SocketAddr, client: Client) {
        self.clients.insert(socket, client);
    }

    fn client_read(&mut self, auther_addr: SocketAddr) {
        if let Some(auther) = self.clients.get_mut(&auther_addr) {
            let mut buffer = [0;64];
            let bytes: Vec<_> = match auther.connection.read(&mut buffer){
                Ok(0) => {
                    println!("Client {ip}:{port} disconnected", ip = auther_addr.ip(), port = auther_addr.port());
                    self.clients.remove(&auther_addr);
                    return;
                },
                Ok(n) => buffer[0..n].iter().cloned().filter(|x| *x >= 32).collect(),
                Err(err) => {
                    if err.kind() != std::io::ErrorKind::WouldBlock {
                        eprintln!("Couldn't read data from client: {err}");
                        self.clients.remove(&auther_addr);
                    }
                    return;
                }
            };

            let text = if let Ok(text) = std::str::from_utf8(&bytes) {
                text
            } else {
                return;
            };

            if text.len() > 0 {println!("Message: {}", text)};

            for (client_addr, client) in self.clients.iter_mut() {
                if auther_addr != *client_addr {
                    let _ = writeln!(client.connection, "{text}");
                }
            }
        }
    }

    fn update(&mut self) {
        let sockets: Vec<SocketAddr> = self.clients.keys().cloned().collect();
        for socket in sockets {
            self.client_read(socket);
        }
    }
}

fn main() -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{PORT}"))?;

    listener.set_nonblocking(true)?;

    let mut server = Server::new();

    thread::spawn(move || {
        loop {
            let mut input = String::new();
            if let Err(e) = stdin().read_line(&mut input) {
                eprintln!("Error reading input: {}", e);
                break;
            }
            let input = input.trim();
            if input == "quit" {
                println!("Exiting");
                std::process::exit(0);
            }
        }
    });

    loop {
        match listener.accept() {
            Ok((stream, socket)) => {
                if let Err(err) = stream.set_nonblocking(true) {
                    eprintln!("Couldn't connect to socket: {err}");
                    break;
                }
                server.insert_client(socket, Client::new(stream));
                println!("New Client {ip}:{port}", ip = socket.ip(), port = socket.port());
            },
            Err(err) => if err.kind() != std::io::ErrorKind::WouldBlock {
                eprintln!("err: {err}");
            }
        }

        server.update();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    Ok(())

    
}
