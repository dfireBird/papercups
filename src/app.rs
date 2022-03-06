use std::{
    fmt::Debug,
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
    widgets::Clear,
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
        widgets::{self, DialogBox, DialogBoxType, DialogCallback, DialogState},
    },
    ChannelMessage, DEFAULT_PORT,
};

type Client = TcpStream;

/// The main data structure which contains all the necessary variables for `papercups`
/// frontend
#[derive(Debug)]
pub struct App {
    client: Option<Client>,
    mode: AppMode,
    state: State,
    rx: Receiver<ChannelMessage>,
    tx: Sender<ChannelMessage>,
    id: u32,
}

impl App {
    pub fn new(rx: Receiver<ChannelMessage>, tx: Sender<ChannelMessage>) -> Self {
        Self {
            client: None,
            mode: AppMode::Standard,
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
                    if let None = self.client {
                        let msg = format!(
                            "A connection request has been made by {ip} \nDo you want to accept?"
                        );
                        (self.mode, self.state.dialog_state) = decision_dialog_box(
                            msg,
                            Box::new(move |app| {
                                app.tx.send(ChannelMessage::ConnectAccept)?;
                                if let None = app.client {
                                    if let Some(stream) = initiate_client(app.id, ip)? {
                                        app.client = Some(stream);
                                    } // TODO: Should log error when client sent an wrong handshake
                                }
                                Ok(())
                            }),
                            Box::new(|app| {
                                app.tx.send(ChannelMessage::Disconnect)?;
                                Ok(())
                            }),
                        );
                    } else {
                        self.tx.send(ChannelMessage::ConnectAccept)?;
                    }
                }
                ChannelMessage::Message(msg) => {
                    self.state.messages.push((MsgType::Recv, msg.message()))
                }
                ChannelMessage::File(file) => {
                    let msg =
                        "A file has been sent by the peer \nDo you want to save it?".to_string();
                    (self.mode, self.state.dialog_state) = decision_dialog_box(
                        msg,
                        Box::new(move |app| {
                            file.save();
                            Ok(app
                                .state
                                .messages
                                .push((MsgType::Recv, "sent a file".to_string())))
                        }),
                        Box::new(|_| Ok(())),
                    );
                }
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
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Min(15),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            f.render_widget(widgets::connection_status_message(&self.client), chunks[0]);
            f.render_widget(widgets::message_box(&self.state.messages), chunks[1]);
            f.render_widget(widgets::input_box(&self.state.input), chunks[2]);

            if let AppMode::DialogBox(msg, d_type) = &self.mode {
                let centered_area = widgets::centered_rect(35, 20, f.size());
                f.render_widget(Clear, centered_area);
                f.render_stateful_widget(
                    DialogBox::new(msg.to_string(), *d_type),
                    centered_area,
                    &mut self.state.dialog_state.as_mut().unwrap(),
                );
            }
        })?;
        Ok(())
    }

    fn handle_input(&mut self, events: &Events) -> Result<bool> {
        if let Event::Input(input) = events.next()? {
            match input.code {
                KeyCode::Enter => {
                    match self.mode {
                        AppMode::Standard => {
                            let input: String = self.state.input.drain(..).collect();

                            let mut splits = vec![&input[0..1]];
                            splits.append(&mut input[1..].split_whitespace().collect());

                            match Command::try_parse_from(splits) {
                                Ok(command) => match command.subcmd {
                                    Commands::Connect(c) => {
                                        let ip = IpAddr::from_str(&c.ip)?;
                                        if let Some(stream) = initiate_client(self.id, ip)? {
                                            self.client = Some(stream)
                                        } else {
                                            let msg = "Not able to connect successfully. \nThe peer sent a wrong handshake.";
                                            (self.mode, self.state.dialog_state) =
                                                info_dialog_box(msg.to_string());
                                        }
                                    }
                                    Commands::Disconnect => {
                                        if let Some(_) = self.client {
                                            self.tx.send(ChannelMessage::Disconnect)?;
                                            self.client = None;
                                        }
                                    }
                                    Commands::File(file) => {
                                        let path = Path::new(&file.path);
                                        if let Some(client) = &self.client {
                                            if let Some(file) = File::new(path) {
                                                let mut client = client;
                                                client.write(&file.to_bytes())?;
                                                self.state.messages.push((
                                                    MsgType::Sent,
                                                    "sent a file".to_string(),
                                                ));
                                            } // TODO: handle None case
                                        } // TODO: handle not connected case
                                    }
                                    Commands::Quit => {
                                        return Ok(true);
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
                        AppMode::DialogBox(..) => {
                            let answer = self.state.dialog_state.take().unwrap();
                            if answer.is_yes() {
                                (answer.yes_fn)(self)?;
                            } else {
                                (answer.no_fn)(self)?;
                            }
                            self.mode = AppMode::Standard;
                        }
                    }
                }
                KeyCode::Left => {
                    if let AppMode::DialogBox(..) = self.mode {
                        self.state.dialog_state.as_mut().unwrap().toggle();
                    }
                }
                KeyCode::Right => {
                    if let AppMode::DialogBox(..) = self.mode {
                        self.state.dialog_state.as_mut().unwrap().toggle();
                    }
                }
                KeyCode::Char(c) if c == 'd' && input.modifiers == KeyModifiers::CONTROL => {
                    return Ok(true);
                }
                KeyCode::Char(c) if c == 'c' && input.modifiers == KeyModifiers::CONTROL => {
                    return Ok(true);
                }
                KeyCode::Char(c) => {
                    if let AppMode::Standard = self.mode {
                        self.state.input.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if let AppMode::Standard = self.mode {
                        self.state.input.pop();
                    }
                }
                _ => (),
            }
        };
        Ok(false)
    }
}

/// AppMode specifies which mode App is currently in
#[derive(Debug)]
enum AppMode {
    Standard,
    DialogBox(String, DialogBoxType),
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
    dialog_state: Option<DialogState>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            dialog_state: None,
        }
    }
}

#[derive(Debug, Parser)]
struct Command {
    #[clap(subcommand)]
    subcmd: Commands,
}

#[derive(Debug, Parser)]
enum Commands {
    Connect(ConnectCommand),
    Disconnect,
    File(FileCommnad),
    Quit,
}

#[derive(Debug, Parser)]
struct ConnectCommand {
    ip: String,
}

#[derive(Debug, Parser)]
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

fn decision_dialog_box(
    msg: String,
    yes_fn: DialogCallback,
    no_fn: DialogCallback,
) -> (AppMode, Option<DialogState>) {
    (
        AppMode::DialogBox(msg, DialogBoxType::Decision),
        Some(DialogState::new(yes_fn, no_fn)),
    )
}

fn info_dialog_box(msg: String) -> (AppMode, Option<DialogState>) {
    (
        AppMode::DialogBox(msg, DialogBoxType::Info),
        Some(DialogState::default()),
    )
}
