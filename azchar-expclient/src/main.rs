use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let input = if let Some(input) = std::env::args().nth(1) {
        input
    } else {
        "{\"Roll\":\"2d10+1d4+6\"}".to_string()
    };

    let mut stream = TcpStream::connect("127.0.0.1:55555").unwrap();
    stream.write_all(input.as_bytes()).expect("Can't send");
    stream.flush().expect("Can't flush");

    // Receive.
    let mut input = vec![0; 10 * 1024 * 1024];
    input = match stream.read(&mut input) {
        Err(e) => {
            println!("Bad stream from {:?} because {:?}.", stream.peer_addr(), e);
            return;
        }
        Ok(n) => input[0..n].to_vec(),
    };
    let out = String::from_utf8_lossy(&input);
    println!("{}", out);
}
// {\"Roll\":\"2d10dl1mx10+1d4+6\"}
