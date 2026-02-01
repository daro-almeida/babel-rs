use crate::core::protocol::{Protocol, ProtocolHandle, ProtocolId};
use std::any::Any;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventId(pub u16);

pub trait Request: Any + Send + 'static {
    fn as_any(&self) -> &dyn Any;
}

pub type RequestHandlerFn = Box<dyn Fn(&mut dyn Any, &dyn Request, ProtocolId, &ProtocolHandle) + Send + Sync>;
