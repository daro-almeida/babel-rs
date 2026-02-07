use std::any::{Any, TypeId};
use std::collections::HashSet;
use crate::core::event::{Event, IPCEvent};
use crate::core::protocol::{Protocol, ProtocolId, ProtocolRuntime};
use anyhow::anyhow;
use dashmap::{DashMap, Entry};
use std::sync::Arc;
use log::warn;
use crate::protocol::ProtocolHandlers;

pub struct Babel {
    protocols: DashMap<ProtocolId, ProtocolRuntime>,
    notification_subscriptions: DashMap<TypeId, HashSet<ProtocolRuntime>>,
    // timer_manager: TimerManager,
    // channel_manager: ChannelManager,
}

impl Babel {
    pub fn new() -> Arc<Self> {
        Arc::new(Babel {
            protocols: DashMap::new(),
            notification_subscriptions: DashMap::new(),
        })
    }

    pub fn register_protocol(self: &Arc<Self>, protocol: impl Protocol + ProtocolHandlers + 'static) -> anyhow::Result<()> {
        let e = self.protocols.entry(protocol.id());
        match e {
            Entry::Occupied(v) => Err(anyhow!("ProtocolId conflict: {} <-> {}", v.get().name, protocol.name())),
            Entry::Vacant(v) => {
                v.insert(ProtocolRuntime::new(protocol, self.clone()));
                Ok(())
            }
        }
    }

    pub fn start(&mut self) {

    }

    pub fn send_request(&self, from: ProtocolId, to: ProtocolId, ipc: impl IPCEvent) {
        self.send_single_ipc(to, Event::Request(from, Box::new(ipc)));
    }

    pub fn send_reply(&self, from: ProtocolId, to: ProtocolId, ipc: impl IPCEvent) {
        self.send_single_ipc(to, Event::Reply(from, Box::new(ipc)));
    }

    pub fn send_single_ipc(&self, to: ProtocolId, event: Event) {
        if let Some(runtime) = self.protocols.get(&to) {
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
