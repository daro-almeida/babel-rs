use babel_macros::{protocol, Request};
use babel_rs::protocol::{Protocol, ProtocolHandle, ProtocolId};

#[derive(Debug, Request)]
pub struct HelloRequest {
    pub subject: String,
}

#[derive(Debug, Request)]
pub struct GoodbyeRequest {
    pub reason: String,
}

pub struct HelloProtocol {
    id: ProtocolId,
}

impl Protocol for HelloProtocol {
    fn id(&self) -> ProtocolId {
        self.id
    }

    fn name(&self) -> &str {
        "HelloProtocol"
    }

    fn init(&mut self) {
        println!("Init");
    }
}

pub trait HelloProtocolSpec: Protocol {
    fn on_hello(&mut self, req: &HelloRequest, sender: ProtocolId, handle: &ProtocolHandle);
    fn on_goodbye(&mut self, req: &GoodbyeRequest, sender: ProtocolId, handle: &ProtocolHandle);
}

#[protocol]
impl HelloProtocolSpec for HelloProtocol {
    fn on_hello(&mut self, req: &HelloRequest, sender: ProtocolId, handle: &ProtocolHandle) {
        println!("Hello, {}!", req.subject);
    }

    fn on_goodbye(
        &mut self,
        GoodbyeRequest { reason }: &GoodbyeRequest,
        sender: ProtocolId,
        handle: &ProtocolHandle,
    ) {
        println!("Goodbye: {}", reason);
    }
}

fn main() {

}