use crate::core::protocol::{ProtocolHandle, ProtocolId};
use crate::internal::message::AnyMessage;
use std::any::Any;
use std::net::SocketAddr;
use std::sync::Arc;
pub(crate) use crate::internal::ipc::{Notification, Ipc};

pub struct RequestEvent {
    pub source: ProtocolId,
    pub destination: ProtocolId,
    pub ipc: Box<dyn Ipc>
}

pub struct ReplyEvent {
    pub source: ProtocolId,
    pub destination: ProtocolId,
    pub ipc: Box<dyn Ipc>
}

pub struct NotificationEvent {
    pub source: ProtocolId,
    pub ipc: Arc<dyn Notification>
}

pub struct MessageEvent {
    pub source: ProtocolId,
    pub from: SocketAddr,
    pub destination: ProtocolId,
    pub to: SocketAddr,
    pub message: Arc<dyn AnyMessage>
}

pub enum Event {
    Request(RequestEvent),
    Reply(ReplyEvent),
    Notification(NotificationEvent), 
    Message(MessageEvent),
    Shutdown,
}

pub type IpcHandlerFn = Box<dyn Fn(&mut dyn Any, &dyn Ipc, ProtocolId, ProtocolHandle)>;
pub type MessageHandlerFn = Box<dyn Fn(&mut dyn Any, &dyn AnyMessage, SocketAddr, ProtocolId, ProtocolHandle)>;
pub type ShutdownHandlerFn = Box<dyn Fn(&mut dyn Any, ProtocolHandle)>;
