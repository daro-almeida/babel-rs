use std::sync::Arc;
use crate::core::protocol::{Protocol, ProtocolId, ProtocolRuntime};
use anyhow::anyhow;
use dashmap::{DashMap, Entry};

pub struct Babel {
    protocols: DashMap<ProtocolId, ProtocolRuntime>,
    // notification_manager: NotificationManager,
    // timer_manager: TimerManager,
    // channel_manager: ChannelManager,
}

impl Babel {
    pub fn new() -> Self {
        Babel {
            protocols: DashMap::new()
        }
    }

    pub fn start(&mut self) {}

    pub fn register_protocol(&mut self, protocol: impl Protocol + 'static) -> anyhow::Result<()> {
        let e = self.protocols.entry(protocol.id());
        match e {
            Entry::Occupied(_) => Err(anyhow!("Protocol Id conflict")),
            Entry::Vacant(v) => {
                v.insert(ProtocolRuntime::new(protocol, Arc::new(self)));
                Ok(())
            }
        }
    }
}
