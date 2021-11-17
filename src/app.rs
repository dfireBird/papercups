use std::{
    io::{Read, Stdout, Write},
    net::{IpAddr, TcpStream},
    sync::mpsc::{Receiver, Sender},
    thread,
};

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyModifiers};
use tui::{backend::CrosstermBackend, Terminal};

use crate::{
    network::{
        protocol::{Handshake, Serializable},
        Server,
    },
    ui::{
        self,
        events::{Event, Events},
    },
    ChannelMessage, DEFAULT_PORT,
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
            let should_quit = self.handle_input(&events)?;
            if should_quit {
                break Ok(());
            }
        }
    }

    fn recv_from_channel(&mut self) -> Result<()> {
        for message in self.rx.try_iter() {
            match message {
                ChannelMessage::ConnectRequest(id, ip) => {
                    self.tx.send(ChannelMessage::ConnectAccept)?;
                    if let None = self.client {
                        if let Some(stream) = initiate_client(self.id, ip)? {
                            self.client = Some(stream)
                        } // TODO: Should display error message when client sent an wrong handshake
                    }
                }
                ChannelMessage::Message(msg) => {
                    self.state.messages.push((MsgType::Recv, msg.message()))
                }
                ChannelMessage::File(file) => file.save(),
                ChannelMessage::Disconnect => self.client = None,
                _ => (),
            };
        }
        Ok(())
    }

    fn draw_ui(&mut self, term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        term.draw(|f| {})?;
        Ok(())
    }

    fn handle_input(&mut self, events: &Events) -> Result<bool> {
        if let Event::Input(input) = events.next()? {
            match input.code {
                KeyCode::Char(c) if c == 'd' && input.modifiers == KeyModifiers::CONTROL => {
                    return Ok(true);
                }
                KeyCode::Char(c) if c == 'c' && input.modifiers == KeyModifiers::CONTROL => {
                    return Ok(true);
                }
                KeyCode::Char(c) => {
                    self.state.input.push(c);
                }
                KeyCode::Backspace => {
                    self.state.input.pop();
                }
                _ => (),
            }
        };
        Ok(false)
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

fn initiate_client(id: u32, ip: IpAddr) -> Result<Option<TcpStream>> {
    let mut stream = TcpStream::connect((ip, DEFAULT_PORT))?;

    let handshake = Handshake::new(id);
    stream.write(&handshake.to_bytes())?;

    let mut buf = [0u8; 9];
    stream.read_exact(&mut buf)?;
    let recv_handshake =
        Handshake::from_bytes(buf.to_vec()).context("Malformed Handshake message")?;

    if recv_handshake == handshake {
        Ok(Some(stream))
    } else {
        Ok(None)
    }
}
