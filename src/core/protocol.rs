use crate::core::event::{Event, IPCEvent, IPCHandlerFn};
use log::{debug, warn};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProtocolId(pub u16);

impl Display for ProtocolId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
pub struct ProtocolRuntime {
    proto_event_sender: mpsc::Sender<Event>,
}

impl ProtocolRuntime {
    pub fn new(proto_event_sender: mpsc::Sender<Event>) -> Self {
        Self {
            proto_event_sender,
        }
    }

    pub fn send_event(&self, event: Event) {
        self.proto_event_sender
            .send(event)
            .expect("Protocol event channel closed unexpectedly");
    }

    pub fn spawn_protocol_thread(
        mut protocol: Box<dyn Protocol>,
        proto_event_receiver: mpsc::Receiver<Event>,
        babel_event_sender: mpsc::Sender<Event>,
    ) {
        thread::spawn(move || {
            let protocol_id = protocol.id();

            let request_handlers = protocol.get_request_handlers();
            let reply_handlers = protocol.get_reply_handlers();
            let notif_handlers = protocol.get_notification_handlers();

            let handle = ProtocolHandle::new(protocol_id, babel_event_sender);

            protocol.init(&handle);

            loop {
                let protocol_any: &mut dyn Any = &mut protocol;
                match proto_event_receiver.recv() {
                    Ok(Event::Request(from, _, event)) => Self::handle_ipc_event(
                        protocol_any,
                        &*event,
                        from,
                        &handle,
                        &request_handlers,
                    ),
                    Ok(Event::Reply(from, _, event)) => Self::handle_ipc_event(
                        protocol_any,
                        &*event,
                        from,
                        &handle,
                        &reply_handlers,
                    ),
                    Ok(Event::Notification(from, event)) => Self::handle_ipc_event(
                        protocol_any,
                        &*event,
                        from,
                        &handle,
                        &notif_handlers,
                    ),
                    Ok(Event::Message) | Ok(Event::Channel) => todo!(),
                    Ok(Event::Shutdown) => {
                        debug!("Protocol {} shutting down", protocol.name());
                        break;
                    }
                    Err(_) => {
                        panic!("Protocol {} channel closed", protocol.name());
                    }
                }
            }
        });
    }

    fn handle_ipc_event(
        protocol_any: &mut dyn Any,
        event: &dyn IPCEvent,
        from: ProtocolId,
        handle: &ProtocolHandle,
        handlers_map: &HashMap<TypeId, IPCHandlerFn>,
    ) {
        let type_id = event.type_id();

        if let Some(handler) = handlers_map.get(&type_id) {
            handler(protocol_any, event, from, handle)
        } else {
            warn!("No handler for IPC {:?}", type_id);
        }
    }
}

pub struct ProtocolHandle {
    id: ProtocolId,
    babel_event_sender: mpsc::Sender<Event>,
}

impl ProtocolHandle {
    pub fn new(id: ProtocolId, babel_event_sender: mpsc::Sender<Event>) -> Self {
        Self {
            id,
            babel_event_sender,
        }
    }

    pub fn send_request(&self, to: ProtocolId, request: impl IPCEvent) {
        self.babel_event_sender
            .send(Event::Request(self.id, to, Box::new(request)))
            .expect("Babel event channel closed");
    }

    pub fn send_reply(&self, to: ProtocolId, reply: impl IPCEvent) {
        self.babel_event_sender
            .send(Event::Reply(self.id, to, Box::new(reply)))
            .expect("Babel event channel closed");
    }

    pub fn notify(&self, notif: impl IPCEvent) {
        self.babel_event_sender
            .send(Event::Notification(self.id, Arc::new(notif)))
            .expect("Babel event channel closed");
    }
}

pub trait ProtocolInit {
    fn id(&self) -> ProtocolId;
    fn name(&self) -> &str {
        // default to implementing struct name
        std::any::type_name::<Self>() //.rsplit("::").next().unwrap()
    }
    fn init(&mut self, handle: &ProtocolHandle);
}

pub trait ProtocolHandlers: ProtocolInit {
    fn get_request_handlers(&self) -> HashMap<TypeId, IPCHandlerFn>;
    fn get_reply_handlers(&self) -> HashMap<TypeId, IPCHandlerFn>;
    fn get_subscriptions(&self) -> Vec<TypeId>;
    fn get_notification_handlers(&self) -> HashMap<TypeId, IPCHandlerFn>;

    //fn get_message_handlers() -> HashMap<TypeId, todo!()>;
    //fn get_channel_event_handlers() -> HashMap<TypeId, todo!()>;
    //fn get_shutdown_handler() -> HashMap<TypeId, todo!()>;
}

pub trait Protocol: ProtocolInit + ProtocolHandlers + Send + 'static {}
impl<T: Protocol> Protocol for T {}