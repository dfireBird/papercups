use rand::Rng;

mod app;
mod network;

pub const DEFAULT_PORT: u16 = 42069;

/// Payload used in channels between UI thread and server thread
pub enum ChannelMessage {
    // TODO: Add payload to enum variants
    ConnectRequest,
    ConnectAccept,
    Message,
    File,
    Disconnect,
}

pub fn generate_id() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}
