mod app;
mod network;
mod ui;

use std::net::IpAddr;

use rand::Rng;

use crate::network::protocol::{File, Message};

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
    rng.gen()
}
