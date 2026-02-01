use crate::core::babel::Babel;
use crate::core::event::{EventId, Request, RequestHandlerFn};
use babel_macros::{Request, protocol};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProtocolId(pub u16);

pub struct ProtocolRuntime {
    protocol: Box<dyn Protocol>,
    //event_queue: BinaryHeap<Event>,

    //channels: HashMap<ChannelId, Channel>,
    //timer_handlers: HashMap<EventId, Fn>,
    request_handlers: HashMap<EventId, RequestHandlerFn>,
    protocol_handle: ProtocolHandle, //reply_handlers: HashMap<EventId, Fn>,
                                     //notification_handlers: HashMap<EventId, Fn>,
}

impl ProtocolRuntime {
    pub fn new(protocol: impl Protocol + 'static, babel_instance: Arc<Babel>) -> Self {
        Self {
            protocol: Box::new(protocol),
            request_handlers: HashMap::new(),
            protocol_handle: ProtocolHandle::new(babel_instance),
        }
    }

    pub fn start() {
        // spawn thread
    }
}

pub struct ProtocolHandle {
    babel_instance: Arc<Babel>,
}

impl ProtocolHandle {
    pub fn new(babel_instance: Arc<Babel>) -> Self {
        Self { babel_instance }
    }

    pub fn send_request(&self, to: ProtocolId, request: &dyn Request) {
        //self.babel_instance.
        todo!()
    }
}

pub trait Protocol: Send + 'static {
    fn id(&self) -> ProtocolId;
    fn name(&self) -> &str;
    fn init(&mut self);
}

pub trait ProtocolHandlers {
    fn get_request_handlers() -> HashMap<TypeId, RequestHandlerFn>;
}
