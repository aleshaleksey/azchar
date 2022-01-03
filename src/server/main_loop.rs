//! Here we deal with the main loop.
use crate::database::root_db::LoadedDbs;

pub struct MainLoop {
    pub(super) dbs: Option<LoadedDbs>,
}
