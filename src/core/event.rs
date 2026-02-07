use crate::core::protocol::{ProtocolHandle, ProtocolId};
use std::any::Any;
use std::sync::Arc;

pub trait IPCEvent: Any + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
}

pub enum Event {
    Request(ProtocolId, Box<dyn IPCEvent>),
    Reply(ProtocolId, Box<dyn IPCEvent>),
    Notification(ProtocolId, Arc<dyn IPCEvent>),
    Message,
    Channel,
    Shutdown
}

pub type IPCHandlerFn = Box<dyn Fn(&mut dyn Any, &dyn IPCEvent, ProtocolId, &ProtocolHandle)>;
