use crate::{
    commands::invite::Invite, commands::register::Register, state::transactions::Transaction,
};

#[derive(Clone)]
pub struct InboundInit {
    pub reg: Register,
    pub msg: String,
}

#[derive(Clone)]
pub struct OutboundInit {
    pub inv: Invite,
    pub msg: String,
    pub transaction: Transaction,
}
