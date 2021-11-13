use std::{
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::mpsc::{Receiver, Sender},
};

use crate::{ChannelMessage, DEFAULT_PORT};

pub mod protocol;

/// Structure for the state of `papercups` backend or server stack
#[derive(Debug)]
pub struct Server {
    server: TcpListener,
    peer_stream: Option<TcpStream>,
    rx: Receiver<ChannelMessage>,
    tx: Sender<ChannelMessage>,
}

impl Server {
    pub fn new(rx: Receiver<ChannelMessage>, tx: Sender<ChannelMessage>) -> Self {
        Self {
            server: TcpListener::bind((Ipv4Addr::UNSPECIFIED, DEFAULT_PORT)).unwrap(),
            peer_stream: None,
            rx,
            tx,
        }
    }
}
