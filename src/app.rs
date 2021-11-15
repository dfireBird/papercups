use std::{
    io::Stdout,
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
    thread,
};

use anyhow::Result;
use tui::{backend::CrosstermBackend, Terminal};

use crate::{
    network::Server,
    ui::{self, events::Events},
    ChannelMessage,
};

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

    pub fn start(mut self, server: Server) -> Result<()> {
        let handle = thread::spawn(|| server.start_server());
        let mut term = ui::initialize_term()?;

        self.start_ui_loop(&mut term)?;

        ui::deinitialize_term(term)?;
        handle
            .join()
            .expect("Couldn't join on the associated thread")
    }

    fn start_ui_loop(&mut self, term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        let events = Events::new();

        loop {
            self.recv_from_channel()?;
            self.draw_ui(term)?;
            self.handle_input(&events);
        }
    }

    fn recv_from_channel(&mut self) -> Result<()> {
        todo!()
    }

    fn draw_ui(&mut self, term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        todo!()
    }

    fn handle_input(&mut self, events: &Events) {
        todo!()
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
