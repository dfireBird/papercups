use std::{
    io::{Read, Stdout, Write},
    net::{IpAddr, TcpStream},
    path::Path,
    str::FromStr,
    sync::mpsc::{Receiver, Sender},
    thread,
};

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{KeyCode, KeyModifiers};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

use crate::{
    network::{
        protocol::{File, Handshake, Message, Serializable},
        Server,
    },
    ui::{
        self,
        events::{Event, Events},
        widgets,
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
        thread::spawn(|| server.start_server());
        let mut term = ui::initialize_term()?;

        self.start_ui_loop(&mut term)?;

        ui::deinitialize_term(term)?;
        Ok(())
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
        term.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Min(15), Constraint::Length(3)].as_ref())
                .split(f.size());

            f.render_widget(widgets::message_box(&self.state.messages), chunks[0]);
            f.render_widget(widgets::input_box(&self.state.input), chunks[1]);
        })?;
        Ok(())
    }

    fn handle_input(&mut self, events: &Events) -> Result<bool> {
        if let Event::Input(input) = events.next()? {
            match input.code {
                KeyCode::Enter => {
                    let input: String = self.state.input.drain(..).collect();
                    match Command::try_parse_from(input.split_whitespace()) {
                        Ok(command) => match command.subcmd {
                            Commands::Connect(c) => {
                                let ip = IpAddr::from_str(&c.ip)?;
                                if let Some(stream) = initiate_client(self.id, ip)? {
                                    self.client = Some(stream)
                                } // TODO: Should display error message when client sent an wrong handshake
                            }
                            Commands::Disconnect => {
                                if let Some(_) = self.client {
                                    self.tx.send(ChannelMessage::Disconnect)?;
                                    self.client = None;
                                }
                            }
                            Commands::File(file) => {
                                let path = Path::new(&file.path);
                                if let Some(file) = File::new(path) {
                                    if let Some(client) = &self.client {
                                        let mut client = client;
                                        client.write(&file.to_bytes())?;
                                    } // TODO: handle not connected case
                                } // TODO: handle None case
                            }
                        },
                        Err(_) => {
                            if let Some(client) = &self.client {
                                let mut client = client;
                                let msg = Message::new(input);
                                client.write(&msg.to_bytes())?;
                                self.state.messages.push((MsgType::Sent, msg.message()));
                            } // TODO: handle not connected case
                        }
                    }
                }
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
pub enum MsgType {
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

#[derive(Parser)]
struct Command {
    #[clap(subcommand)]
    subcmd: Commands,
}

#[derive(Parser)]
enum Commands {
    Connect(ConnectCommand),
    Disconnect,
    File(FileCommnad),
}

#[derive(Parser)]
struct ConnectCommand {
    ip: String,
}

#[derive(Parser)]
struct FileCommnad {
    path: String,
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
