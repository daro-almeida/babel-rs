use crate::protocol::ProtocolId;
use rkyv::api::high::HighSerializer;
use rkyv::api::low::LowValidator;
use rkyv::bytecheck::CheckBytes;
use rkyv::de::Unpool;
use rkyv::rancor::Strategy;
use rkyv::ser::allocator::ArenaHandle;
use rkyv::util::AlignedVec;
use rkyv::{Archive, Deserialize, Serialize};
use std::any::Any;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
pub struct MessageId(pub u16);

pub trait Message:
    Archive<
        Archived: for<'a> CheckBytes<LowValidator<'a, rkyv::rancor::Error>>
                      + Deserialize<Self, Strategy<Unpool, rkyv::rancor::Error>>,
    > + for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, rkyv::rancor::Error>>
    + Any
    + Send
where
    Self: Sized,
{
    const ID: MessageId;
}

pub trait AnyMessage: Any + Send {
    fn id(&self) -> MessageId;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
    fn as_any(&self) -> &dyn Any;
}

impl<T: Message> AnyMessage for T {
    fn id(&self) -> MessageId {
        T::ID
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Archive, Serialize, Deserialize)]
pub struct BabelMessage {
    pub id: MessageId,
    pub inner_message_bytes: Box<[u8]>,
    pub source: ProtocolId,
    pub destiny: ProtocolId,
}
