use babel_macros::{protocol, reply_handler, request_handler, IPC};
use babel_rs::protocol::{Protocol, ProtocolHandle, ProtocolId, ProtocolInit};

#[derive(Debug, IPC)]
pub struct Hello {
    pub subject: String,
}

#[derive(Debug, IPC)]
pub struct Goodbye {
    pub reason: String,
}

pub struct HelloProtocol {
    id: ProtocolId,
}

#[protocol]
impl HelloProtocol {
    #[request_handler]
    fn on_hello(
        &mut self,
        Hello { subject }: &Hello,
        _source_proto: ProtocolId,
        _handle: &ProtocolHandle,
    ) {
        println!("Hello, {}!", subject);
    }

    #[reply_handler]
    fn on_goodbye(
        &mut self,
        Goodbye { reason }: &Goodbye,
        _source_proto: ProtocolId,
        _handle: &ProtocolHandle,
    ) {
        println!("Goodbye: {}", reason);
    }
}

fn main() {}
