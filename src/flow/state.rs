use crate::{composer::registration::Register, commands::invite::Invite};

#[derive(Clone)]
pub struct InboundInit {
    pub reg: Register,
    pub msg: String,
}


#[derive(Clone)]
pub struct OutboundInit {
    pub inv: Invite,
}