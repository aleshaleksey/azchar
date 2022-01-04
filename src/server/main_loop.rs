//! Here we deal with the main loop.
use crate::database::root_db::LoadedDbs;
use crate::error::ma;

use std::net::TcpListener;

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

    pub(crate) fn run(&mut self) {
        unimplemented!()
    }
}
