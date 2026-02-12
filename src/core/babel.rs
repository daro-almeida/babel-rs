use crate::core::event::{Event, IPCEvent};
use crate::core::protocol::{Protocol, ProtocolId, ProtocolRuntime};
use anyhow::anyhow;
use dashmap::DashMap;
use log::warn;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, mpsc};
use std::thread;

pub struct BabelInit {
    protocols: HashMap<ProtocolId, Box<dyn Protocol>>,
}

impl BabelInit {
    pub fn new() -> Self {
        BabelInit {
            protocols: HashMap::new(),
        }
    }

    pub fn register_protocol(&mut self, protocol: impl Protocol) -> anyhow::Result<()> {
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

    pub fn start(self) -> Arc<Babel> {
        let mut proto_channer_pairs = HashMap::new();
        let runtimes = DashMap::new();
        let notification_subscriptions: DashMap<TypeId, Vec<ProtocolRuntime>> = DashMap::new();
        let mut babel_event_receivers = Vec::new();

        for (&id, protocol) in self.protocols.iter() {
            let (proto_event_sender, proto_event_receiver) = mpsc::channel();
            let (babel_proto_event_sender, babel_proto_event_receiver) = mpsc::channel();

            proto_channer_pairs.insert(id, (babel_proto_event_sender, proto_event_receiver));
            babel_event_receivers.push(babel_proto_event_receiver);

            let runtime = ProtocolRuntime::new(proto_event_sender);
            runtimes.insert(id, runtime.clone());

            for sub in protocol.get_subscriptions() {
                notification_subscriptions
                    .entry(sub)
                    .or_default()
                    .push(runtime.clone());
            }
        }

        let babel = Arc::new(Babel {
            runtimes,
            notification_subscriptions,
        });

        for babel_proto_event_receiver in babel_event_receivers {
            Babel::start_protocol_event_listener(babel.clone(), babel_proto_event_receiver);
        }

        for (id, protocol) in self.protocols {
            let (babel_proto_event_sender, proto_event_receiver) =
                proto_channer_pairs.remove(&id).unwrap();
            ProtocolRuntime::spawn_protocol_thread(
                protocol,
                proto_event_receiver,
                babel_proto_event_sender,
            );
        }

        babel
    }
}

pub struct Babel {
    runtimes: DashMap<ProtocolId, ProtocolRuntime>,
    notification_subscriptions: DashMap<TypeId, Vec<ProtocolRuntime>>,
    // timer_manager: TimerManager,
    // channel_manager: ChannelManager,
}

impl Babel {
    pub fn shutdown(&self) {
        todo!()
    }
    
    fn start_protocol_event_listener(self: Arc<Self>, babel_proto_event_receive: Receiver<Event>) {
        thread::spawn(move || {
            loop {
                match babel_proto_event_receive.recv() {
                    Ok(event) => match event {
                        Event::Request(_, to, _) | Event::Reply(_, to, _) => {
                            self.send_single_ipc(to, event)
                        }
                        Event::Notification(from, ipc) => self.send_notification(from, ipc),
                        Event::Message => todo!(),
                        Event::Channel => todo!(),
                        Event::Shutdown => todo!(),
                    },
                    Err(_) => panic!("Protocol event listener closed unexpectedly"),
                }
            }
        });
    }

    fn send_single_ipc(&self, to: ProtocolId, event: Event) {
        if let Some(runtime) = self.runtimes.get(&to) {
            runtime.value().send_event(event);
        } else {
            warn!("Protocol with id {} not registered", to);
        }
    }

    fn send_notification(&self, from: ProtocolId, ipc_arc: Arc<dyn IPCEvent>) {
        if let Some(subscribers) = self.notification_subscriptions.get(&ipc_arc.type_id()) {
            for runtime in subscribers.value() {
                runtime.send_event(Event::Notification(from, ipc_arc.clone()));
            }
        }
    }
}
