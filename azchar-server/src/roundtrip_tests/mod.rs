//! This module exists to run a servant and client.
//! This will use the ExpClient, which is somewhat limited.
//! Currently the messagesize is limited to 10MB.

use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use std::net::{Shutdown, TcpStream};

use crate::{Mode, MainLoop};
use crate::requests::{Request, Response};

mod test_library;

/// A structure which deletes all traces of the directory it writes.
/// It's a little rudimentary.
pub(self) struct Frame {
    // Directory where the system will be stored.
    dir: PathBuf,
    // Thread where the server is run.
    thread: Option<JoinHandle<()>>,
    // Client TcpStream.
    address: String,
}

// Each time a frame sends a message, it expects a certain response.
pub(self) enum FrameReply {
    Success(Response),
    Fail(String),
}

impl Drop for Frame {
    fn drop(&mut self) {
        // First drop the client. It should be the easiest part.
        // This is a little defensive, but there we go.
        let mut stream = TcpStream::connect(&self.address).expect("Bad address?");
        let shutdown_song = serde_json::to_string(&Request::Shutdown).expect("Yes.");
        send_message(shutdown_song, &mut stream);
        stream.shutdown(Shutdown::Both).expect("Couldn't shut stream");
        drop(stream);

        // Try to drop the directory where we kept all the data.
        if let Err(e1) = std::fs::remove_dir_all(&self.dir) {
           if let Err(e2) = std::fs::remove_dir_all(&self.dir) {
               println!("Could not remove own directory: {:?}, {:?}", e1, e2);
           }
       }
    }
}

impl Frame {
    /// Create the testframe.
    fn create(dir: PathBuf, address: &str) -> Self {
        let addr = address.to_string();
        let mut frame = Frame {
            dir,
            thread: None,
            address: address.to_string(),
        };
        let handle = thread::spawn(move || {
            let mode = Mode::Client;

            match MainLoop::create_with_connection(&addr) {
                    Ok(mut ml) => ml.run(mode),
                    Err(e) => println!("{}", e),
            }
        });
        frame.thread = Some(handle);
        // We need to sleep because the server needs a moment to start up.
        // before we can begin accepting signals.
        std::thread::sleep(std::time::Duration::from_millis(10));
        frame
    }

    /// Send a request and get a reply.
    /// This is done in such a way, so we can send a request, and see if
    /// the reply from the server is what we would expect or not.
    /// This then gives us control over when to end the test case.
    fn send_and_receive(&mut self, message: Request) -> FrameReply {
        let mut stream = TcpStream::connect(&self.address).expect("Bad address?");

        let letter = serde_json::to_string(&message).expect("invalid");
        let letter_home = send_message(letter, &mut stream);

        match serde_json::from_str(&letter_home) {
            Ok(r) => FrameReply::Success(r),
            Err(e) => FrameReply::Fail(format!("{:?}", e)),
        }
    }
}

/// TODO: Figure out how to import `expclient` properly, so we don't have to use this.
pub fn send_message(message: String, stream: &mut TcpStream) -> String {
    use std::io::{Write, Read};

    stream.write_all(message.as_bytes()).expect("Can't send");
    stream.flush().expect("Can't flush");

    // Receive.
    let mut input = vec![0; 10 * 1024 * 1024];
    input = match stream.read(&mut input) {
        Err(e) => {
            println!("Bad stream from {:?} because {:?}.", stream.peer_addr(), e);
            return String::new();
        }
        Ok(n) => input[0..n].to_vec(),
    };
    let out = String::from_utf8_lossy(&input);
    println!("{}", out);
    out.to_owned().to_string()
    // String::new()
}
