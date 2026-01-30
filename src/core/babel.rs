use std::collections::HashMap;
use crate::core::protocol::{Protocol, ProtocolId};

pub struct Babel {
    protocols: HashMap<ProtocolId, Protocol>,
    //events: HashMap<>,

}