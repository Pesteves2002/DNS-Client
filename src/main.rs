use std::{env, error::Error, io::Error as IoError, net::UdpSocket};

use crate::structs::message::Message;

mod structs;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        return Err(IoError::other(
            "usage: cargo run <domain> <TYPE> <CLASS> <NS>",
        ).into());
    }

    let domain = args.get(1).unwrap();
    let qtype = args.get(2).unwrap();
    let qclass = args.get(3).unwrap();
    let name_server = args.get(4).unwrap();

    let packet = Message::create_query(domain, qtype, qclass)?.to_bytes();

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.send_to(&packet, format!("{name_server}:53"))?;

    let mut buf = [0u8; 512];
    let (len, _) = socket.recv_from(&mut buf)?;

    let response = Message::from_bytes(&buf, len)?;
    println!("{}", response);

    Ok(())
}
