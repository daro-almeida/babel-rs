mod app;
mod gossip;
mod membership;

use std::process::exit;
use std::thread;
use log::LevelFilter;
use crate::app::ChatApp;
use crate::gossip::FloodGossip;
use babel::babel::BabelBuilder;
use crate::membership::FullMembership;

fn main() -> anyhow::Result<()> {
    let address = "127.0.0.1:3000".parse()?;
    let babel = BabelBuilder::new()
        .with_logging(LevelFilter::Info)
        .register_protocol(ChatApp::new())?
        .register_protocol(FloodGossip::new(address, 2))?
        .register_protocol(FullMembership::new(address, 2))?
        .start();
    ctrlc::set_handler(move || {
        babel.shutdown();
        exit(0);
    })?;
    thread::park();
    Ok(())
}
