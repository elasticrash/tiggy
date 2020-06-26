extern crate tokio;
use tokio::net::UdpSocket;

#[derive(Copy, Clone)]
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

trait Merge {
    fn generate_sip(&self) -> String;
}

impl Merge for SIP<'_> {
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
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let username = "1615391830:441164961072";
    let password = "****";
    let sip_server = "****:5060";

    let mut socket = UdpSocket::bind("0.0.0.0:5060").await.unwrap();

    let command = SIP {
        command: "REGISTER sip:192.168.137.8 SIP/2.0",
        content_length: "Content-Length: 0",
        to: "To: sip:1615391830:441164961072@192.168.137.8",
        from: "From: sip:1615391830:441164961072@192.168.137.8",
        contact: "Contact: sip:1615391830:441164961072@185.28.212.48;transport=UDP",
        cseq: "CSeq: 445 REGISTER",
        call_id: "Call-ID: b6f928e6a981e32d24c98ee789575f09@192.168.137.8",
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
    println!("[{}] - {:?}", line!(), full_message.split_at(amt).0);

    Ok(())
}
