use std::any::Any;

pub trait Ipc: Any + Send {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
    fn as_any(&self) -> &dyn Any;
}

pub trait Notification: Ipc + Sync {}
