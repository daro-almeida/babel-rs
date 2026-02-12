mod app;
mod gossip;
mod membership;

use std::thread;
use crate::app::ChatApp;
use crate::gossip::FloodGossip;
use babel::babel::BabelInit;
use crate::membership::FullMembership;

fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    let mut init = BabelInit::new();
    init.register_protocol(ChatApp::new())?;
    init.register_protocol(FloodGossip::new("127.0.0.1:3000".parse().unwrap(), 2))?;
    init.register_protocol(FullMembership::new("127.0.0.1:3000".parse().unwrap(), 2))?;
    // TODO handle babel or Ctrl+C
    thread::park();
    Ok(())
}
