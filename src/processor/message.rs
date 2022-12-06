use std::sync::mpsc::{self, Receiver, Sender};

pub trait Execute {
    fn execute();
}

pub fn setup_processor<T>() -> (Sender<T>, Receiver<T>) {
    mpsc::channel()
}

pub struct Message {
    pub message_type: MessageType,
    pub bind: char,
    pub content: Option<String>,
}

pub enum MessageType {
    MenuCommand,
}

impl Message {
    pub fn new(message_type: MessageType, bind: char, content: Option<String>) -> Self {
        Self {
            message_type,
            bind,
            content,
        }
    }
}
