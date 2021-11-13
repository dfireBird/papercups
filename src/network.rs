use std::{
    io::{Read, Write},
    net::{Ipv4Addr, Shutdown, TcpListener, TcpStream},
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use anyhow::{Context, Result};

use crate::{
    network::protocol::{ProtocolMessage, Serializable},
    ChannelMessage, DEFAULT_PORT,
};

use self::protocol::Handshake;

pub mod protocol;

/// Strcuture containing the state of `papercups` backend or server stack
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

    pub fn start_server(mut self) -> Result<()> {
        loop {
            match &mut self.peer_stream {
                None => {
                    self.connect_peer()?;
                }
                Some(peer) => {
                    if let Ok(msg) = self.rx.try_recv() {
                        if let ChannelMessage::Disconnect = msg {
                            peer.shutdown(Shutdown::Both)?;
                            self.peer_stream = None;
                            continue;
                        }
                    }

                    let mut buf = [0u8; 8];
                    if peer.peek(&mut buf)? != 0 {
                        let length = u32::from_be_bytes(buf[4..8].try_into().context(
                            "Malformed Header Recieved: Lenght is not valid 4-byte (32-bit) number",
                        )?);

                        let mut data = vec![0u8; 8 + length as usize];
                        peer.read_exact(&mut data)?;

                        match ProtocolMessage::from_bytes(data)? {
                            ProtocolMessage::Message(msg) => {
                                self.tx.send(ChannelMessage::Message(msg))
                            }
                            ProtocolMessage::File(file) => self.tx.send(ChannelMessage::File(file)),
                        }?;
                    } else {
                        self.peer_stream = None;
                        self.tx.send(ChannelMessage::Disconnect)?;
                        continue;
                    }
                }
            }
        }
    }

    /// Accepts a peer and send message to UI thread for user confirmation on connecting to peer.
    /// Initiates handshake after confirmation from the user and updates server state.
    fn connect_peer(&mut self) -> Result<()> {
        let (mut peer, addr) = match self.server.accept() {
            Ok((peer, addr)) => (peer, addr),
            Err(_) => return Ok(()),
        };

        peer.set_read_timeout(Some(Duration::from_secs(120)))?;
        let mut buffer = [0; 9];
        peer.read(&mut buffer)?;
        peer.set_read_timeout(None)?;

        let handshake = Handshake::from_bytes(buffer.to_vec())?;
        self.tx
            .send(ChannelMessage::ConnectRequest(handshake.id(), addr.ip()))?;
        if let ChannelMessage::ConnectAccept = self.rx.recv()? {
            peer.write(&handshake.to_bytes())?;
            self.peer_stream = Some(peer);
        }

        Ok(())
    }
}
