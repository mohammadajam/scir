use std::io::{self, Write, Read};
use std::net::TcpStream;
use std::str;
use std::thread;
use std::time::Duration;

const IP: &str = "192.168.0.109";
const PORT: u32 = 4000;

fn main() -> io::Result<()> {
    let server_addr = format!("{}:{}", IP, PORT);
    let mut stream = TcpStream::connect(server_addr)?;
    stream.set_nonblocking(true)?;
    println!("Connected to the server");

    let mut input_stream = stream.try_clone()?;

    thread::spawn(move || {
        loop {
            let mut input = String::new();
            if let Err(e) = io::stdin().read_line(&mut input) {
                eprintln!("Error reading input: {}", e);
                break;
            }

            let input = input.trim();

            if input == "quit" {
                println!("Exiting");
                std::process::exit(0);
            }

            if !input.is_empty() {
                if let Err(e) = writeln!(input_stream, "{}", input) {
                    eprintln!("Error sending data: {}", e);
                    break;
                }
                input_stream.flush().unwrap();
            }
        }
    });

    loop {
        let mut buffer = [0; 512];
        match stream.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                let response = str::from_utf8(&buffer[..bytes_read]).unwrap_or("Invalid UTF-8");
                println!("Message: {}", response);
            }
            Ok(_) => {
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                eprintln!("Error reading from server: {}", e);
                break;
            }
        }
    }

    Ok(())
}

