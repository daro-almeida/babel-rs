mod app;
mod gossip;
mod membership;

use crate::app::ChatApp;
use crate::gossip::FloodGossip;
use crate::membership::FullMembership;
use babel::babel::BabelBuilder;
use log::LevelFilter;
use std::process::exit;
use std::thread;

fn main() -> anyhow::Result<()> {
    let address = "127.0.0.1:3000".parse()?;
    let babel = BabelBuilder::new()
        .with_logging(LevelFilter::Trace)
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
