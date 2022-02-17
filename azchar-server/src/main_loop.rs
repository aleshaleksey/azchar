//! Here we deal with the main loop.
use super::Mode;
use crate::requests::{Request, Response};
use azchar_database::root_db::LoadedDbs;
use azchar_error::ma;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const START: &str = "AZCHARREQUEST__";
const END: &str = "__ENDAZCHARREQUEST";

const ESC: [&str; 26] = [
    "%3A", "%3C", "%3E", "%5B", "%5D", "%7B", "%7D", "%2B", "%2F", "%2C", "%3B", "%3D", "%3F",
    "%5C", "%5E", "%7C", "%7E", "%23", "%20", "%24", "%25", "%26", "%40", "%60", "%22", "%27",
];
const ESC2: [&str; 26] = [
    ":", "<", ">", "[", "]", "{", "}", "+", "/", ",", ";", "=", "?", "\\", "^", "|", "~", "#", " ",
    "$", "%", "&", "@", "`", "\"", "'",
];

pub struct MainLoop {
    /// This represents the local connection to the system.
    pub(super) dbs: Option<LoadedDbs>,
    /// This represents the TCP stream.
    pub(super) stream: TcpListener,
}

impl MainLoop {
    pub(crate) fn create_with_connection(address: &str) -> Result<Self, String> {
        let stream = TcpListener::bind(address).map_err(ma)?;
        Ok(Self { dbs: None, stream })
    }

    pub(crate) fn run(&mut self, mode: Mode) {
        let stream = &mut self.stream;
        for stream in stream.incoming() {
            let res = match (stream, mode) {
                (Ok(s), Mode::Client) => Self::handle_stream_as_client(s, &mut self.dbs),
                (Ok(s), _) => Self::handle_stream_as_http(s, &mut self.dbs),
                (Err(e), _) => Err(format!("Incoming error: {:?}", e)),
            };
            match res {
                Ok(true) => {}
                Err(e) => println!("Error: {:?}", e),
                Ok(false) => break,
            }
        }
    }

    fn handle_stream_as_client(
        mut s: TcpStream,
        dbs: &mut Option<LoadedDbs>,
    ) -> Result<bool, String> {
        let peer = match s.peer_addr() {
            Ok(addr) => format!("{}", addr),
            _ => "Unknown Sender".to_owned(),
        };
        let mut input = vec![0; 1024 * 1024];
        input = match s.read(&mut input) {
            Err(e) => {
                return Err(format!("Bad stream from {:?} because {:?}.", peer, e));
            }
            Ok(n) => input[0..n].to_vec(),
        };
        // println!("About to handle input: {}", String::from_utf8_lossy(&input));
        let mut echo = match String::from_utf8(input) {
            Ok(s) => s,
            Err(e) => format!("Client receiving error: {:?}", e),
        };
        let ln = echo.chars().count();

        let res = if ln == 0 {
            "Invalid input mofo!!!".to_string()
        } else {
            for (a, b) in ESC.iter().zip(ESC2.iter()) {
                echo = echo.replace(a, b);
            }
            let req = Request::convert(&echo);
            // println!("{:?}", req);
            match req.execute(dbs) {
                Ok(Response::Shutdown) => return Ok(false),
                Ok(r) => serde_json::to_string(&r),
                Err(e) => serde_json::to_string(&Response::Err(echo, ma(e))),
            }
            .unwrap()
        };
        send_and_flush(&mut s, &res, &peer)?;
        Ok(true)
    }

    fn handle_stream_as_http(
        mut s: TcpStream,
        dbs: &mut Option<LoadedDbs>,
    ) -> Result<bool, String> {
        let peer = match s.peer_addr() {
            Ok(addr) => format!("{}", addr),
            _ => "Unknown Sender".to_owned(),
        };
        let mut input = vec![0; 1024 * 1024];
        input = match s.read(&mut input) {
            Err(e) => {
                return Err(format!("Bad stream from {:?} because {:?}.", peer, e));
            }
            Ok(n) => input[0..n].to_vec(),
        };
        let mut echo = String::from_utf8(input).unwrap();
        // println!("{}", echo);
        let ln = echo.chars().count();

        echo = echo.split_once(END).unwrap_or(("", "")).0.to_owned();
        echo = echo.split_once(START).unwrap_or(("", "")).1.to_owned();
        let ln2 = echo.chars().count();

        let res = if ln == ln2 || ln2 == 0 {
            "Invalid input mofo!!!".to_string()
        } else {
            for (a, b) in ESC.iter().zip(ESC2.iter()) {
                echo = echo.replace(a, b);
            }
            let req = Request::convert(&echo);
            // println!("{:?}", req);
            match req.execute(dbs) {
                Ok(Response::Shutdown) => return Ok(false),
                Ok(r) => serde_json::to_string(&r),
                Err(e) => serde_json::to_string(&Response::Err(echo, ma(e))),
            }
            .unwrap()
        };
        let ret = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\nContent: {}",
            res.len() + 8,
            res
        );
        send_and_flush(&mut s, &ret, &peer)?;
        Ok(true)
    }
}

fn send_and_flush(s: &mut TcpStream, msg: &str, peer: &str) -> Result<(), String> {
    if let Err(e) = s.write(msg.as_bytes()) {
        return Err(format!("POST Can't reply to {:?} because {:?}.", peer, e));
    }
    if let Err(e) = s.flush() {
        return Err(format!("Can't flush {:?} because {:?}.", peer, e));
    }
    Ok(())
}
