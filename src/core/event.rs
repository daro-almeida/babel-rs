use crate::core::protocol::{ProtocolHandle, ProtocolId};
use std::any::Any;
use std::sync::Arc;

pub trait IPCEvent: Any + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
}

pub enum Event {
    Request(ProtocolId, ProtocolId, Box<dyn IPCEvent>), //from, to, event
    Reply(ProtocolId, ProtocolId, Box<dyn IPCEvent>), //from, to, event
    Notification(ProtocolId, Arc<dyn IPCEvent>), //from, event
    Message,
    Channel,
    Shutdown
}

pub type IPCHandlerFn = Box<dyn Fn(&mut dyn Any, &dyn IPCEvent, ProtocolId, ProtocolHandle)>;
pub type ShutdownHandlerFn = Box<dyn Fn(&mut dyn Any, ProtocolHandle)>;
