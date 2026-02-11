use std::{env, io::Error, net::UdpSocket};

use crate::structs::message::Message;

mod structs;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        return Err(Error::other("usage: cargo run <domain> <TYPE> <CLASS>"));
    }

    let domain = args.get(1).unwrap();
    let qtype = args.get(2).unwrap();
    let qclass = args.get(3).unwrap();

    let packet = Message::create_query(domain, qtype, qclass).to_bytes();

    let socket = UdpSocket::bind("0.0.0.0:0")?;

    socket.send_to(&packet, "8.8.8.8:53")?;

    let mut buf = [0u8; 512];
    let (len, _) = socket.recv_from(&mut buf)?;

    let mut data = &buf[..len];

    let response = Message::from_bytes(&mut data);
    println!("{}", response);

    Ok(())
}

#[cfg(test)]
mod tests {
    use dns_parser::QueryType;

    use super::*;

    #[test]
    fn create_query() {
        let domain = "tomase.pt";
        let qtype = "A";
        let qclass = "IN";

        let packet = Message::create_query(domain, qtype, qclass);

        let mut builder = dns_parser::Builder::new_query(packet.header.id, true);

        builder.add_question(domain, false, QueryType::A, dns_parser::QueryClass::IN);

        let reference = builder.build().unwrap();

        assert_eq!(packet.to_bytes(), reference)
    }
}
