use babel::protocol::{ProtocolHandle, ProtocolId, ProtocolInit};
use babel_macros::{message_handler, protocol, Ipc, Message};
use log::debug;
use rand::rngs::SmallRng;
use rand::seq::{IndexedRandom, IteratorRandom};
use rkyv::{Archive, Deserialize, Serialize};
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

#[derive(Debug, Archive, Serialize, Deserialize, Message)]
#[message_id(1)]
pub struct ShuffleMessage {
    pub sample: HashSet<SocketAddr>,
}

#[derive(Debug, Archive, Serialize, Deserialize, Message)]
#[message_id(2)]
pub struct ShuffleReplyMessage {
    pub sample: HashSet<SocketAddr>,
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

    fn init(&mut self, _handle: ProtocolHandle) {
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
            rng: rand::make_rng(),
        }
    }

    // TODO shuffle timer handler

    #[message_handler]
    fn upon_shuffle(
        &mut self,
        msg: ShuffleMessage,
        from: SocketAddr,
        _source: ProtocolId,
        _handle: ProtocolHandle,
    ) {
        debug!("Received {:?} from {}", msg, from);

        let mut subset = Self::random_subset_excluding(
            &mut self.rng,
            self.membership.iter(),
            from,
            self.subset_size,
        );

        subset.insert(self.myself);
        let reply = ShuffleReplyMessage {sample: subset};
        // TODO send reply on IN
        debug!("Sent {:?} to {}", reply, from);
        for h in &msg.sample {
            if *h != self.myself && !self.membership.contains(h) && !self.pending.contains(h) {
                self.pending.insert(*h);
                // TODO open connection to h
            }
        }
    }

    #[message_handler]
    fn upon_shuffle_reply(
        &mut self,
        msg: ShuffleReplyMessage,
        from: SocketAddr,
        _source: ProtocolId,
        _handle: ProtocolHandle,
    ) {
        debug!("Received {:?} from {}", msg, from);
        for h in &msg.sample {
            if *h != self.myself && !self.membership.contains(h) && !self.pending.contains(h) {
                self.pending.insert(*h);
                // TODO open connection to h
            }
        }
    }

    // TODO message fail events

    // TODO channel events

    fn random_subset_excluding<'a, T: PartialEq + Clone+ 'a + Eq + std::hash::Hash>(
        rng: &mut SmallRng,
        it: impl IteratorRandom<Item = &'a T>,
        e: T,
        n: usize,
    ) -> HashSet<T> {
        it.filter(|x| **x != e).cloned().sample(rng, n).into_iter().collect()
    }
}
