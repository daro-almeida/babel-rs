use crate::gossip::{BroadcastRequest, DeliverNotification, GOSSIP_ID};
use babel::protocol::{ProtocolHandle, ProtocolId, ProtocolInit};
use babel_macros::{notification_handler, protocol};
use log::info;
use std::process::exit;
use std::{io, thread};

pub struct ChatApp;

pub const CHAT_APP_ID: ProtocolId = ProtocolId(301);

impl ProtocolInit for ChatApp {
    fn id(&self) -> ProtocolId {
        CHAT_APP_ID
    }

    fn init(&mut self, handle: ProtocolHandle) {
        thread::spawn(move || {
            let mut line = String::new();
            let stdin = io::stdin();
            loop {
                stdin.read_line(&mut line).unwrap();
                if line == "quit" {
                    exit(0)
                }
                handle.send_request(GOSSIP_ID, BroadcastRequest { msg: line.clone() })
            }
        });
    }
}

#[protocol]
impl ChatApp {
    pub fn new() -> Self {
        Self {}
    }
    
    #[notification_handler]
    fn upon_deliver(
        &mut self,
        DeliverNotification { msg, via, n_hops }: &DeliverNotification,
        _: ProtocolId,
        _: ProtocolHandle,
    ) {
        info!("Received via {}: {} - {} hops", via, msg, n_hops);
    }
}
