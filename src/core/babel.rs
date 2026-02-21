use crate::core::protocol::{Protocol, ProtocolId, ProtocolRuntime};
use crate::internal::event::{
    Notification, Event, MessageEvent, NotificationEvent, ReplyEvent, RequestEvent,
};
use anyhow::anyhow;
use dashmap::DashMap;
use log::warn;
use std::any::{Any, TypeId};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc};
use std::thread;

pub struct BabelBuilder {
    protocols: HashMap<ProtocolId, Box<dyn Protocol>>,
    log_level: Option<log::LevelFilter>,
}

impl BabelBuilder {
    pub fn new() -> Self {
        BabelBuilder {
            protocols: HashMap::new(),
            log_level: None,
        }
    }

    pub fn with_logging(mut self, level: log::LevelFilter) -> Self {
        self.log_level = Some(level);
        self
    }

    pub fn register_protocol(mut self, protocol: impl Protocol) -> anyhow::Result<Self> {
        match self.protocols.entry(protocol.id()) {
            Entry::Occupied(v) => Err(anyhow!(
                "ProtocolId conflict: {:?} <-> {:?}",
                v.get().type_id(),
                protocol.type_id()
            )),
            Entry::Vacant(v) => {
                v.insert(Box::new(protocol));
                Ok(self)
            }
        }
    }

    pub fn start(self) -> Arc<Babel> {
        if let Some(level) = self.log_level {
            env_logger::Builder::new()
                .filter_level(level)
                .try_init()
                .ok();
        }

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
    pub fn thread_join_handles() {
        todo!()
    }

    pub fn shutdown(&self) {
        for e in self.runtimes.iter() {
            e.value().send_event(Event::Shutdown)
        }
    }

    fn start_protocol_event_listener(self: Arc<Self>, babel_proto_event_receive: Receiver<Event>) {
        thread::spawn(move || {
            loop {
                match babel_proto_event_receive.recv() {
                    Ok(event) => match event {
                        Event::Request(RequestEvent { destination, .. })
                        | Event::Reply(ReplyEvent { destination, .. }) => {
                            if let Some(runtime) = self.runtimes.get(&destination) {
                                runtime.value().send_event(event);
                            } else {
                                warn!(
                                    "Sending Ipc: Protocol with id {} not registered",
                                    destination
                                );
                            }
                        }
                        Event::Notification(NotificationEvent { source, notification: ipc }) => {
                            self.send_notification(source, ipc)
                        }
                        Event::Message(MessageEvent { .. }) => {
                            // TODO tokio spawn serialize and send message
                        }
                        Event::Shutdown => unreachable!(), // TODO maybe discern events that are received here from the ones received in the protocol
                    },
                    Err(_) => panic!("Protocol event listener closed unexpectedly"),
                }
            }
        });
    }

    fn send_notification(&self, source: ProtocolId, ipc_arc: Arc<dyn Notification>) {
        if let Some(subscribers) = self.notification_subscriptions.get(&ipc_arc.type_id()) {
            for runtime in subscribers.value() {
                runtime.send_event(Event::Notification(NotificationEvent {
                    source,
                    notification: ipc_arc.clone(),
                }));
            }
        }
    }
}
