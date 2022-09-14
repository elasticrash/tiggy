use super::transactions::Transaction;

pub struct Dialog {
    pub diag_type: DialogType,
    pub call_id: String,
    pub local_tag: String,
    pub remote_tag: String,
    pub transactions: Vec<Transaction>,
}

pub enum DialogType {
    Inbound,
    Outbound,
}


pub struct Dialogs {
    pub state: Vec<Dialog>
}