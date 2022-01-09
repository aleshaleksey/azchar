//! This deals with a websocket type system.
use crate::requests::{Request, Response};
use azchar_database::root_db::LoadedDbs;
use azchar_error::ma;

use websocket::server::NoTlsAcceptor;
use websocket::sync::Server;
use websocket::{Message, OwnedMessage};

pub struct WsMainLoop {
    /// This represents the local connection to the system.
    pub(super) dbs: Option<LoadedDbs>,
    /// This represents the Websocket stream.
    pub(super) stream_addr: String,
}

impl WsMainLoop {
    pub(crate) fn create_with_conn(address: &str) -> Result<Self, String> {
        Ok(Self {
            dbs: None,
            stream_addr: address.to_string(),
        })
    }

    // Can only be run in WS mode.
    pub(crate) fn run(mut self) {
        loop {
            match self.run_inner() {
                Ok(_) => println!("Stream  processed successfully."),
                Err(e) => println!("Stream process failed: {:?}", e),
            }
        }
    }

    fn run_inner(&mut self) -> Result<(), String> {
        let dbs = &mut self.dbs;
        let mut stream = Server::<NoTlsAcceptor>::bind(&self.stream_addr).map_err(ma)?;
        let u = stream.accept().map_err(ma)?;
        let cli = u.accept().map_err(ma)?;
        println!("Accepting connection: {:?}", cli.peer_addr());
        let (mut receiver, mut sender) = cli.split().unwrap();

        for m in receiver.incoming_messages() {
            println!("m:{:?}", m);
            match m {
                Ok(OwnedMessage::Close(_d)) => {
                    println!("Close.");
                    continue;
                }
                Ok(OwnedMessage::Text(t)) => {
                    let res = match Request::convert(t.clone()).execute(dbs) {
                        Ok(r) => serde_json::to_string(&r),
                        Err(_) => serde_json::to_string(&Response::Err(t)),
                    }
                    .map_err(ma)?;
                    let m = Message::text(&res);
                    sender.send_message(&m).map_err(ma)?;
                }
                Err(e) => return Err(ma(e)),
                _ => {}
            }
        }
        Ok(())
    }
}
