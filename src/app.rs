use std::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};

use crate::ChannelMessage;

type Client = TcpStream;

/// The main data structure which contains all the necessary variables for `papercups`
/// frontend
#[derive(Debug)]
pub struct App {
    client: Option<Client>,
    state: State,
    rx: Receiver<ChannelMessage>,
    tx: Sender<ChannelMessage>,
    id: u32,
}

impl App {
    pub fn new(rx: Receiver<ChannelMessage>, tx: Sender<ChannelMessage>) -> Self {
        Self {
            client: None,
            state: State::default(),
            id: crate::generate_id(),
            rx,
            tx,
        }
    }
}

/// Classifies the messages based on whether is received or sent
#[derive(Debug)]
enum MsgType {
    Recv,
    Sent,
}

#[derive(Debug)]
struct State {
    messages: Vec<(MsgType, String)>,
    input: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
        }
    }
}
