use crate::core::event::{Event, IPCEvent};
use crate::core::protocol::{Protocol, ProtocolId, ProtocolRuntime};
use crate::protocol::ProtocolHandlers;
use anyhow::anyhow;
use dashmap::{DashMap};
use log::warn;
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::marker::PhantomData;
use std::sync::{mpsc, Arc};
use std::sync::mpsc::Receiver;
use std::thread;

struct SettingUp;
struct Started;

pub struct Babel<State> {
    state: State,
    protocols: HashMap<ProtocolId, Box<dyn Protocol>>,
    runtimes: DashMap<ProtocolId, ProtocolRuntime>,
    notification_subscriptions: DashMap<TypeId, Vec<ProtocolRuntime>>,
    // timer_manager: TimerManager,
    // channel_manager: ChannelManager,
}

impl Babel<SettingUp> {
    pub fn new() -> Self {
        Babel {
            state: SettingUp,
            protocols: HashMap::new(),
            runtimes: DashMap::new(),
            notification_subscriptions: DashMap::new(),
        }
    }

    pub fn register_protocol(
        &mut self,
        protocol: impl Protocol,
    ) -> anyhow::Result<()> {
        match self.protocols.entry(protocol.id()) {
            Entry::Occupied(v) => Err(anyhow!(
                "ProtocolId conflict: {} <-> {}",
                v.get().name(),
                protocol.name()
            )),
            Entry::Vacant(v) => {
                v.insert(Box::new(protocol));
                Ok(())
            }
        }
    }

    fn start_protocol_event_listener(babel: Arc<Babel<Started>>, babel_proto_event_receive: Receiver<Event>) {
        thread::spawn(move || {
            loop {
                match babel_proto_event_receive.recv() {
                    Ok() => ,
                    Err() => ,
                }
            }
        })
    }

    pub fn start(self) {
        let mut channer_pairs = HashMap::new();
        
        let babel_arc = Arc::new(self);
        for (&id, protocol) in self.protocols.iter() {
            let (proto_event_sender, proto_event_receiver) = mpsc::channel();
            let (babel_proto_event_sender, babel_proto_event_receiver) = mpsc::channel();

            channer_pairs.insert(id, (babel_proto_event_sender, proto_event_receiver));

            Self::start_protocol_event_listener(babel_proto_event_receiver);

            let runtime = ProtocolRuntime::new(proto_event_sender);
            self.runtimes.insert(id, runtime.clone());

            for sub in protocol.get_subscriptions() {
                self.notification_subscriptions.entry(sub).or_default().push(runtime.clone());
            }
        }

        for (id, protocol) in self.protocols {
            let (babel_proto_event_sender, proto_event_receiver) = channer_pairs.remove(&id).unwrap();
            ProtocolRuntime::spawn_protocol_thread(protocol, proto_event_receiver, babel_proto_event_sender);
        }
    }
}

impl Babel<Started> {
    pub fn send_request(&self, from: ProtocolId, to: ProtocolId, ipc: impl IPCEvent) {
        self.send_single_ipc(to, Event::Request(from, Box::new(ipc)));
    }

    pub fn send_reply(&self, from: ProtocolId, to: ProtocolId, ipc: impl IPCEvent) {
        self.send_single_ipc(to, Event::Reply(from, Box::new(ipc)));
    }

    pub fn send_single_ipc(&self, to: ProtocolId, event: Event) {
        if let Some(runtime) = self.runtimes.get(&to) {
            runtime.value().send_event(event);
        } else {
            warn!("Protocol with id {} not registered", to);
        }
    }

    pub fn send_notification(&self, from: ProtocolId, ipc: impl IPCEvent) {
        let ipc_arc = Arc::new(ipc);
        if let Some(subscribers) = self.notification_subscriptions.get(&ipc_arc.type_id()) {
            for runtime in subscribers.value() {
                runtime.send_event(Event::Notification(from, ipc_arc.clone()));
            }
        }
    }
}
