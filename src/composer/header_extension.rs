use rsip::{Header, Headers};

pub trait CustomHeaderExtension {
    fn get_via_header_array(&self) -> Vec<&Header>;
    fn get_record_route_header_array(&self) -> Vec<&Header>;
    fn push_many(&mut self, new_headers:Vec<&Header>);
}

impl CustomHeaderExtension for Headers {
    fn get_via_header_array(&self) -> Vec<&Header> {
        self.iter().filter(|h| match h {
            Header::Via(_) => return true,
            _ => return false,
        }).collect()
    }
    fn get_record_route_header_array(&self) -> Vec<&Header> {
        self.iter().filter(|h| match h {
            Header::RecordRoute(_) => return true,
            _ => return false,
        }).collect()
    }

    fn push_many(&mut self, new_headers: Vec<&Header>) {
        for hd in new_headers {
            self.push(hd.clone().into());
        }
    }
}
