use std::{env, io::Error, net::UdpSocket};

use crate::structs::message::Message;

mod structs;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        return Err(Error::other(
            "usage: cargo run <domain> <TYPE> <CLASS> <NS>",
        ));
    }

    let domain = args.get(1).unwrap();
    let qtype = args.get(2).unwrap();
    let qclass = args.get(3).unwrap();
    let name_server = args.get(4).unwrap();

    let packet = Message::create_query(domain, qtype, qclass).to_bytes();

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.send_to(&packet, format!("{name_server}:53"))?;

    let mut buf = [0u8; 512];
    let (len, _) = socket.recv_from(&mut buf)?;

    let mut data = &buf[..len];

    let response = Message::from_bytes(&mut data);
    println!("{}", response);

    Ok(())
}
