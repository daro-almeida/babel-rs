use std::any::Any;

pub trait Ipc: Any + Send {
    fn as_any(&self) -> &dyn Any;
}

pub trait Notification: Ipc + Sync {}
