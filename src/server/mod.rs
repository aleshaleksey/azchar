//! This module deals with server related things. Namely with the interactions
//! with the outside world.
//! This includes:
//! a) The main loop.
//! b) Requests and responses.
pub(crate) mod main_loop;
pub(super) mod requests;

pub(crate) use main_loop::MainLoop;
