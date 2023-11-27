use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct MpscBase<T> {
    pub event: Option<T>,
    pub override_default_destination: Option<OverwriteDestination>,
    pub exit: bool,
}

#[derive(Debug, Clone)]
pub struct OverwriteDestination {
    pub ip: IpAddr,
    pub port: u16,
}

impl<T> Default for MpscBase<T> {
    fn default() -> Self {
        Self {
            event: None,
            override_default_destination: None,
            exit: false,
        }
    }
}
