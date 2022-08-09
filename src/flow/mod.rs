pub mod inbound;
pub mod outbound;
pub mod state;

pub enum Flow {
    Inbound,
    Outbound,
}
