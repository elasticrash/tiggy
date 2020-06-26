extern crate md5;
extern crate phf;
extern crate rand;
extern crate tokio;
use phf::phf_map;
use phf::Map;
use tokio::net::UdpSocket;

static MAP: Map<&'static str, &'static str> = phf_map! {
      "Content-Length"=>"content_length",
      "To"=>"to",
      "From"=>"from",
      "Contact"=>"contact",
      "CSeq"=>"cseq",
      "Call-ID"=>"call_id",
      "Via"=>"via",
      "User-Agent"=>"user_agent",
      "Allow"=> "allow",
};

#[derive(Copy, Clone, Debug)]
struct SIP<'a> {
    command: &'a str,
    content_length: &'a str,
    to: &'a str,
    from: &'a str,
    contact: &'a str,
    cseq: &'a str,
    call_id: &'a str,
    via: &'a str,
    user_agent: &'a str,
    allow: &'a str,
}

trait SipMessageAttributes {
    fn generate_sip(&self) -> String;
    fn empty() -> Self;
    fn set_by_key(self: &mut Self, key: &str, value: &str);
    fn generate_call_id() -> String;
}

impl SipMessageAttributes for SIP<'_> {
    fn generate_sip(&self) -> String {
        let carrier = "\r\n";
        let msg = vec![
            self.command.to_string(),
            self.content_length.to_string(),
            self.to.to_string(),
            self.from.to_string(),
            self.contact.to_string(),
            self.cseq.to_string(),
            self.call_id.to_string(),
            self.via.to_string(),
            self.user_agent.to_string(),
            self.allow.to_string(),
            carrier.to_string(),
        ];

        return msg.join(carrier);
    }
    fn empty() -> Self {
        return SIP {
            command: "",
            content_length: "Content-Length:",
            to: "To:",
            from: "From:",
            contact: "Contact:",
            cseq: "CSeq:",
            call_id: "Call-ID:",
            via: "Via:",
            user_agent: "User-Agent:",
            allow: "Allow:",
        };
    }
    fn set_by_key<'a>(&mut self, key: &str, value: &str) {
        match MAP.get(key).cloned() {
            Some(data) => {
                if data == "command" {
                    self.command = Box::leak(
                        format!("{}{}", self.command.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
                if data == "content_length" {
                    self.content_length = Box::leak(
                        format!(
                            "{}{}",
                            self.content_length.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "to" {
                    self.to = Box::leak(
                        format!("{}{}", self.to.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
                if data == "from" {
                    self.from = Box::leak(
                        format!("{}{}", self.from.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
                if data == "contact" {
                    self.contact = Box::leak(
                        format!("{}{}", self.contact.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
                if data == "call_id" {
                    self.call_id = Box::leak(
                        format!("{}{}", self.call_id.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
                if data == "via" {
                    self.via = Box::leak(
                        format!("{}{}", self.via.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
                if data == "user_agent" {
                    self.user_agent = Box::leak(
                        format!(
                            "{}{}",
                            self.user_agent.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "allow" {
                    self.allow = Box::leak(
                        format!("{}{}", self.allow.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
            }
            None => {}
        }
    }
    fn generate_call_id() -> String {
        return format!("{:x}", md5::compute(rand::random::<[u8; 16]>()));
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let username = "1615391830:441164961072";
    let password = "E8GBxoC5RTnkBw3AaT+CzjtrYbE=";
    let sip_server = "register.staging.cloudcall.com:5060";
    let ip = get_if_addrs::get_if_addrs().unwrap()[0].addr.ip();
    println!("[{}] - {:?}", line!(), ip.to_string());

    let mut socket = UdpSocket::bind("0.0.0.0:5060").await.unwrap();

    let command = SIP {
        command: &format!("REGISTER sip:{} SIP/2.0", &ip),
        content_length: "Content-Length: 0",
        to: &format!("To: sip:{}@{}", &username, &ip),
        from: &format!("From: sip:{}@{}", &username, &ip),
        contact: &format!("Contact: sip:{}@{};transport=UDP", &username, &ip),
        cseq: "CSeq: 445 REGISTER",
        call_id: &format!("Call-ID:{}@{}", &SIP::generate_call_id(), &ip),
        via: "Via: SIP/2.0/UDP 185.28.212.48;transport=UDP;branch=57ffd673319367006160043a8bad5ab5",
        user_agent: "User-Agent: sippy 0.2.5",
        allow: "Allow: INVITE,CANCEL,BYE,MESSAGE",
    };

    println!("[{}] - {:?}", line!(), command.generate_sip());

    socket
        .send_to(command.generate_sip().as_bytes(), &sip_server)
        .await
        .unwrap();

    let mut buf = [0; 65535];
    let (amt, src) = socket.recv_from(&mut buf).await.unwrap();

    socket
        .send_to(command.generate_sip().as_bytes(), &sip_server)
        .await
        .unwrap();

    let full_message = String::from_utf8_lossy(&buf);
    parser(full_message.split_at(amt).0);
    println!("[{}] - {:?}", line!(), full_message.split_at(amt).0);

    Ok(())
}

fn parser(msg: &str) {
    let carrier = "\r\n";
    let v: Vec<&str> = msg.split(carrier).collect();
    let mut empty_sip = SIP::empty();
    empty_sip.set_by_key("command", &v[0]);

    for i in 1..v.len() {
        let split: Vec<&str> = v[i].split(':').collect();
        let key = split.first().unwrap();
        let value = split.last().unwrap();

        empty_sip.set_by_key(key, value);
    }

    println!("[{}] - {:?}", line!(), empty_sip);
}
