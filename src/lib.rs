mod app;
mod network;
mod ui;

use std::{net::IpAddr, sync::mpsc};

use anyhow::Result;
use rand::Rng;

use crate::app::App;
use crate::network::{
    protocol::{File, Message},
    Server,
};

pub const DEFAULT_PORT: u16 = 42069;

/// Payload used in channels between UI thread and server thread
pub enum ChannelMessage {
    ConnectRequest(u32, IpAddr),
    ConnectAccept,
    Message(Message),
    File(File),
    Disconnect,
}

pub fn generate_id() -> u32 {
    let mut rng = rand::thread_rng();
    loop {
        let id = rng.gen();
        if id != 0 {
            break id;
        }
    }
}

pub fn start_papercups() -> Result<()> {
    let (atx, srx) = mpsc::channel();
    let (stx, arx) = mpsc::channel();
    let app = App::new(arx, atx);
    let server = Server::new(srx, stx);
    app.start(server)
}
