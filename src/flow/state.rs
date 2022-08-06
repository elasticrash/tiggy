use crate::composer::registration::Register;

#[derive(Clone)]
pub struct InboundInit {
    pub reg: Register,
    pub msg: String,
}
