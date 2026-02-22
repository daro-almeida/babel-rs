use crate::membership::{PeerDown, PeerUp};
use babel::internal::ipc::Notification;
use babel::protocol::{ProtocolHandle, ProtocolId, ProtocolInit};
use babel_macros::{
    message_handler, notification_handler, protocol, request_handler, Ipc, Message, Notification,
};
use log::{info, trace};
use rand::prelude::{SliceRandom, SmallRng};
use rkyv::{Archive, Deserialize, Serialize};
use std::cmp::min;
use std::collections::HashSet;
use std::net::SocketAddr;
use uuid::Uuid;

pub const GOSSIP_ID: ProtocolId = ProtocolId(201);

#[derive(Ipc)]
pub struct BroadcastRequest {
    pub msg: String,
}

#[derive(Ipc, Notification)]
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
    rng: SmallRng,
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize, Message)]
#[message_id(0)]
pub struct GossipMessage {
    mid: Uuid,
    round: usize,
    content: String,
}

impl ProtocolInit for FloodGossip {
    fn id(&self) -> ProtocolId {
        GOSSIP_ID
    }

    fn init(&mut self, _handle: ProtocolHandle) {
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
            rng: rand::make_rng(),
        }
    }

    #[request_handler]
    fn upon_broadcast(
        &mut self,
        BroadcastRequest { msg }: BroadcastRequest,
        _: ProtocolId,
        handle: ProtocolHandle,
    ) {
        let mid = Uuid::new_v4();
        let gossip_msg = GossipMessage {
            mid,
            round: 0,
            content: msg,
        };
        self.upon_gossip(gossip_msg, self.myself, self.id(), handle);
    }

    #[message_handler]
    fn upon_gossip(
        &mut self,
        mut msg: GossipMessage,
        from: SocketAddr,
        _: ProtocolId,
        handle: ProtocolHandle,
    ) {
        trace!("Received {:?} from {}", msg, from);
        if self.received.insert(msg.mid) {
            handle.notify(DeliverNotification {
                msg: msg.content.clone(),
                via: from,
                n_hops: msg.round,
            });

            msg.round += 1;

            let mut random_peers = self
                .peers
                .iter()
                .copied()
                .filter(|p| *p != from)
                .collect::<Vec<_>>();
            random_peers.shuffle(&mut self.rng);

            for &peer in random_peers[0..min(self.gossip_size, random_peers.len())].into_iter() {
                //todo!(send_message to host);
                trace!("Sent {:?} to {}", &msg, from);
            }
        }
    }

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
