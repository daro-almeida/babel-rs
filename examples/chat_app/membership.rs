use babel::protocol::{ProtocolHandle, ProtocolId, ProtocolInit};
use babel_macros::{protocol, Ipc};
use rand::rngs::{SmallRng, StdRng};
use std::collections::HashSet;
use std::net::SocketAddr;

pub const MEMBERSHIP_ID: ProtocolId = ProtocolId(102);

#[derive(Ipc)]
pub struct PeerUp {
    pub peer: SocketAddr,
}

#[derive(Ipc)]
pub struct PeerDown {
    pub peer: SocketAddr,
}

pub struct FullMembership {
    myself: SocketAddr,
    membership: HashSet<SocketAddr>,
    pending: HashSet<SocketAddr>,
    subset_size: usize,
    rng: SmallRng,
}

impl ProtocolInit for FullMembership {
    fn id(&self) -> ProtocolId {
        MEMBERSHIP_ID
    }

    fn init(&mut self, handle: ProtocolHandle) {
        todo!("start channel, connect to contact, start periodic shuffle timer, share channel")
    }
}

#[protocol]
impl FullMembership {
    pub fn new(myself: SocketAddr, subset_size: usize) -> Self {
        Self {
            myself,
            membership: HashSet::new(),
            pending: HashSet::new(),
            subset_size,
            rng: rand::make_rng()
        }
    }

    // TODO shuffle timer handler

    // TODO shuffle message handler

    // TODO shuffle reply message handler

    // TODO message fail events

    // TODO channel events
}
