use crate::membership::{PeerDown, PeerUp};
use babel::protocol::{ProtocolHandle, ProtocolId, ProtocolInit};
use babel_macros::{notification_handler, protocol, request_handler, Ipc};
use log::info;
use std::collections::HashSet;
use std::net::SocketAddr;
use uuid::Uuid;

pub const GOSSIP_ID: ProtocolId = ProtocolId(201);

#[derive(Ipc)]
pub struct BroadcastRequest {
    pub msg: String,
}

#[derive(Ipc)]
pub struct DeliverNotification {
    pub msg: String,
    pub via: SocketAddr,
    pub n_hops: usize,
}

pub struct FloodGossip {
    myself: SocketAddr,
    peers: HashSet<SocketAddr>,
    received: HashSet<Uuid>,
    gossip_size: usize,
}

impl ProtocolInit for FloodGossip {
    fn id(&self) -> ProtocolId {
        GOSSIP_ID
    }

    fn init(&mut self, handle: ProtocolHandle) {
        todo!("use shared channel (from membership)")
    }
}

#[protocol]
impl FloodGossip {
    pub fn new(myself: SocketAddr, gossip_size: usize) -> Self {
        Self {
            myself,
            peers: HashSet::new(),
            received: HashSet::new(),
            gossip_size,
        }
    }

    #[request_handler]
    fn upon_broadcast(
        &mut self,
        BroadcastRequest { msg }: &BroadcastRequest,
        _: ProtocolId,
        _: ProtocolHandle,
    ) {
        let message_id = Uuid::new_v4();
        todo!("create gossip message, call gossip msg handler")
    }

    // TODO gossip message handler

    #[notification_handler]
    fn upon_peer_up(&mut self, PeerUp { peer }: &PeerUp, _: ProtocolId, _: ProtocolHandle) {
        self.peers.insert(peer.clone());
        info!(
            "New peer {}, curr view({}): {:?}",
            peer,
            self.peers.len(),
            self.peers
        )
    }

    #[notification_handler]
    fn upon_peer_down(&mut self, PeerDown { peer }: &PeerDown, _: ProtocolId, _: ProtocolHandle) {
        self.peers.insert(peer.clone());
        info!(
            "Bad peer {}, curr view({}): {:?}",
            peer,
            self.peers.len(),
            self.peers
        )
    }
}
