use crate::{commands::invite::Invite, composer::registration::Register};

#[derive(Clone)]
pub struct InboundInit {
    pub reg: Register,
    pub msg: String,
}

#[derive(Clone)]
pub struct OutboundInit {
    pub inv: Invite,
}
