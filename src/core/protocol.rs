use crate::internal::event::{Notification, Ipc, IpcHandlerFn, MessageEvent, MessageHandlerFn, NotificationEvent, ReplyEvent, ShutdownHandlerFn, NotificationHandlerFn};
use crate::internal::event::{Event, RequestEvent};
use log::{debug, warn};
use rkyv::{Archive, Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
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
        Self { proto_event_sender }
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
            let message_handlers = protocol.get_message_handlers();

            let handle = ProtocolHandle::new(protocol_id, babel_event_sender);

            protocol.init(handle.clone());

            loop {
                let protocol_any: &mut dyn Any = &mut protocol;
                match proto_event_receiver.recv() {
                    Ok(Event::Request(RequestEvent { source, ipc, .. })) => Self::handle_ipc_event(
                        protocol_any,
                        ipc,
                        source,
                        handle.clone(),
                        &request_handlers,
                    ),
                    Ok(Event::Reply(ReplyEvent { source, ipc, .. })) => Self::handle_ipc_event(
                        protocol_any,
                        ipc,
                        source,
                        handle.clone(),
                        &reply_handlers,
                    ),
                    Ok(Event::Notification(NotificationEvent { source, notification })) => {
                        let type_id = notification.type_id();

                        if let Some(handler) = notif_handlers.get(&type_id) {
                            handler(protocol_any, &*notification, source, handle.clone())
                        } else {
                            warn!("No handler for received message {:?}", type_id);
                        }
                    }
                    Ok(Event::Message(MessageEvent {
                        source,
                        from,
                        message,
                        ..
                    })) => {
                        let type_id = message.type_id();

                        if let Some(handler) = message_handlers.get(&type_id) {
                            handler(protocol_any, message, from, source, handle.clone())
                        } else {
                            warn!("No handler for received message {:?}", type_id);
                        }
                    }
                    Ok(Event::Shutdown) => {
                        debug!("Protocol {:?} shutting down", protocol.type_id());
                        if let Some(handler) = protocol.get_shutdown_handler() {
                            handler(&mut protocol, handle.clone());
                        } else {
                            debug!("Protocol {:?} has no shutdown handler", protocol.type_id());
                        }
                        break;
                    }
                    Err(_) => {
                        panic!("Protocol {:?} channel closed unexpectedly", protocol.type_id());
                    }
                }
            }
        });
    }

    fn handle_ipc_event(
        protocol_any: &mut dyn Any,
        ipc: Box<dyn Ipc>,
        source: ProtocolId,
        handle: ProtocolHandle,
        handlers_map: &HashMap<TypeId, IpcHandlerFn>,
    ) {
        let type_id = ipc.type_id();

        if let Some(handler) = handlers_map.get(&type_id) {
            handler(protocol_any, ipc, source, handle)
        } else {
            warn!("No handler for received Ipc {:?}", type_id);
        }
    }
}

#[derive(Clone)]
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

    pub fn send_request(&self, destination: ProtocolId, request: impl Ipc) {
        self.babel_event_sender
            .send(Event::Request(RequestEvent {
                source: self.id,
                destination,
                ipc: Box::new(request),
            }))
            .expect("Babel event channel closed");
    }

    pub fn send_reply(&self, destination: ProtocolId, reply: impl Ipc) {
        self.babel_event_sender
            .send(Event::Reply(ReplyEvent {
                source: self.id,
                destination,
                ipc: Box::new(reply),
            }))
            .expect("Babel event channel closed");
    }

    pub fn notify(&self, notif: impl Notification) {
        self.babel_event_sender
            .send(Event::Notification(NotificationEvent {
                source: self.id,
                notification: Arc::new(notif),
            }))
            .expect("Babel event channel closed");
    }
}

pub trait ProtocolInit {
    fn id(&self) -> ProtocolId;
    fn init(&mut self, handle: ProtocolHandle);
}

pub trait ProtocolHandlers: ProtocolInit {
    fn get_request_handlers(&self) -> HashMap<TypeId, IpcHandlerFn>;
    fn get_reply_handlers(&self) -> HashMap<TypeId, IpcHandlerFn>;
    fn get_subscriptions(&self) -> Vec<TypeId>;
    fn get_notification_handlers(&self) -> HashMap<TypeId, NotificationHandlerFn>;
    fn get_message_handlers(&self) -> HashMap<TypeId, MessageHandlerFn>;
    //fn get_channel_event_handlers(&self) -> HashMap<TypeId, todo!()>;
    fn get_shutdown_handler(&self) -> Option<ShutdownHandlerFn>;
}

pub trait Protocol: ProtocolInit + ProtocolHandlers + Send + 'static {}
impl<T: ProtocolInit + ProtocolHandlers + Send + 'static> Protocol for T {}
