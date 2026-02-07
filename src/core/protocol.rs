use crate::core::babel::Babel;
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

pub struct ProtocolRuntime {
    pub name: String,
    event_tx: mpsc::Sender<Event>,
}

impl ProtocolRuntime {
    pub fn new<P: Protocol + ProtocolHandlers>(protocol: P, babel_instance: Arc<Babel>) -> Self {
        let name = protocol.name().to_owned();
        let (event_tx, event_rx) = mpsc::channel();
        Self::spawn_protocol_thread(protocol, event_rx, babel_instance);
        Self { name, event_tx }
    }

    pub fn send_event(&self, event: Event) {
        self.event_tx
            .send(event)
            .expect("Protocol event channel closed unexpectedly");
    }

    fn spawn_protocol_thread<P: Protocol + ProtocolHandlers>(
        mut protocol: P,
        event_rx: mpsc::Receiver<Event>,
        babel: Arc<Babel>,
    ) {
        thread::spawn(move || {
            let protocol_id = protocol.id();

            let req_handlers = protocol.get_request_handlers();
            let reply_handlers = protocol.get_reply_handlers();
            let notif_handlers = protocol.get_notification_handlers();

            let handle = ProtocolHandle::new(protocol_id, babel);

            protocol.init(&handle);

            loop {
                let protocol_any: &mut dyn Any = &mut protocol;
                match event_rx.recv() {
                    Ok(Event::Request(from, event)) => {
                        Self::handle_ipc_event(protocol_any, &*event, from, &handle, &req_handlers)
                    }
                    Ok(Event::Reply(from, event)) => {
                        Self::handle_ipc_event(protocol_any, &*event, from, &handle, &reply_handlers)
                    }
                    Ok(Event::Notification(from, event)) => {
                        Self::handle_ipc_event(protocol_any, &*event, from, &handle, &notif_handlers)
                    }
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
    babel_instance: Arc<Babel>,
}

impl ProtocolHandle {
    pub fn new(id: ProtocolId, babel_instance: Arc<Babel>) -> Self {
        Self { id, babel_instance }
    }

    pub fn send_request(&self, to: ProtocolId, request: impl IPCEvent) {
        self.babel_instance.send_request(self.id, to, request)
    }

    pub fn send_reply(&self, to: ProtocolId, reply: impl IPCEvent) {
        self.babel_instance.send_reply(self.id, to, reply)
    }

    pub fn notify(&self, notif: impl IPCEvent) {
        self.babel_instance.send_notification(self.id, notif)
    }
}

pub trait Protocol: Send + 'static {
    fn id(&self) -> ProtocolId;
    fn name(&self) -> &str {
        // default to implementing struct name
        std::any::type_name::<Self>() //.rsplit("::").next().unwrap()
    }
    fn init(&mut self, handle: &ProtocolHandle);
}

pub trait ProtocolHandlers {
    fn get_request_handlers(&self) -> HashMap<TypeId, IPCHandlerFn>;
    fn get_reply_handlers(&self) -> HashMap<TypeId, IPCHandlerFn>;
    fn get_notification_handlers(&self) -> HashMap<TypeId, IPCHandlerFn>;

    //fn get_message_handlers() -> HashMap<TypeId, todo!()>;
    //fn get_channel_event_handlers() -> HashMap<TypeId, todo!()>;
    //fn get_shutdown_handler() -> HashMap<TypeId, todo!()>;
}
