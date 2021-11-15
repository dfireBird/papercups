use std::{fs, str};

use anyhow::{anyhow, Context, Result};

/// Trait which specifices the strcture can be converted into bytes or from bytes into strcture
pub trait Serializable: Sized {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(data: Vec<u8>) -> Result<Self>;
}

/// Data sent or received through the protcol
#[derive(Debug)]
pub enum ProtocolMessage {
    Message(Message),
    File(File),
}

impl Serializable for ProtocolMessage {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            ProtocolMessage::Message(message) => message.to_bytes(),
            ProtocolMessage::File(file) => file.to_bytes(),
        }
    }

    fn from_bytes(data: Vec<u8>) -> Result<Self> {
        let msg_type = str::from_utf8(&data[0..4])?;
        match msg_type {
            "file" => Ok(Self::File(File::from_bytes(data)?)),
            "chat" => Ok(Self::Message(Message::from_bytes(data)?)),
            _ => Err(anyhow!(
                "Malformed Header Recieved: Invalid message type: {}",
                msg_type
            )),
        }
    }
}

/// Structure for the 'message' type data sent or received through network
#[derive(Debug)]
pub struct Message(String);

impl Message {
    pub fn message(&self) -> String {
        self.0.clone()
    }
}

impl Serializable for Message {
    fn to_bytes(&self) -> Vec<u8> {
        let message = self.0.as_bytes().to_vec();
        let mut data = Vec::from("chat".as_bytes());
        data.append(&mut message.len().to_be_bytes().to_vec());
        data.append(&mut message.to_vec());
        data
    }

    fn from_bytes(data: Vec<u8>) -> Result<Self> {
        Ok(Self(String::from_utf8(data[8..].to_vec()).context(
            "The messeage sent is not a valid UTF-8 string",
        )?))
    }
}

/// Structure for the 'file' type data sent or received through network
#[derive(Debug)]
pub struct File {
    name: String,
    data: Vec<u8>,
}

impl File {
    pub fn save(&self) {
        let mut file_path = dirs::download_dir().unwrap();
        file_path.push(&self.name);
        fs::write(file_path, &self.data).unwrap();
    }
}

impl Serializable for File {
    fn to_bytes(&self) -> Vec<u8> {
        let mut padded_file_name = vec![0u8; 96 - self.name.len()];
        padded_file_name.append(&mut self.name.as_bytes().to_vec());

        let mut data = Vec::from("file".as_bytes());
        data.append(&mut (96 + self.data.len()).to_be_bytes().to_vec());
        data.append(&mut padded_file_name);
        data.append(&mut self.data.clone());
        data
    }

    fn from_bytes(data: Vec<u8>) -> Result<Self> {
        Ok(Self {
            data: data[105..].to_vec(),
            name: String::from_utf8(data[8..104].to_vec())
                .context("Name of the file is not a valid UTF-8 string")?,
        })
    }
}

/// Strcture for handshakes, which sent (or received) before a protocol is established
#[derive(Debug, PartialEq, Eq)]
pub struct Handshake(u32);

impl Handshake {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }
}

impl Serializable for Handshake {
    fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::from("Hello".as_bytes());
        data.append(&mut self.0.to_be_bytes().to_vec());
        data
    }

    fn from_bytes(data: Vec<u8>) -> Result<Self> {
        Ok(Self(u32::from_be_bytes(
            data[5..]
                .try_into()
                .context("Sent ID is not 32-bit (not 4 bytes) number")?,
        )))
    }
}
