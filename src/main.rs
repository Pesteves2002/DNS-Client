use std::{net::UdpSocket, vec};

use rand::Rng;

fn write_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_be_bytes());
}

fn write_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_be_bytes());
}

#[derive(Debug)]
struct Header {
    id: u16,
    flags: u16,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

#[derive(Debug)]
struct DomainName {
    labels: Vec<String>,
}

#[derive(Debug)]
struct Question {
    qname: DomainName,
    qtype: u16,
    qclass: u16,
}

#[derive(Debug)]
struct Answer {
    name: DomainName,
    rtype: u16,
    class: u16,
    ttl: u16,
    rdlength: u16,
    rdata: Vec<u16>,
}

#[derive(Debug)]
struct Message {
    header: Header, // Always present
    question: Vec<Question>,
    answer: Vec<Answer>,
    authority: Vec<Answer>,
    additional: Vec<Answer>,
}

impl Header {
    fn create_query_header() -> Self {
        let mut rng = rand::rng();

        let id = rng.random();

        let flags = 0;

        let qdcount = 1; // 1 question

        let ancount = 0;
        let nscount = 0;
        let arcount = 0;

        Header {
            id,
            flags,
            qdcount,
            ancount,
            nscount,
            arcount,
        }
    }

    fn to_bytes(&self, buf: &mut Vec<u8>) {
        write_u16(buf, self.id);
        write_u16(buf, self.flags);
        write_u16(buf, self.qdcount);
        write_u16(buf, self.ancount);
        write_u16(buf, self.nscount);
        write_u16(buf, self.arcount);
    }
}

impl DomainName {
    fn from_str(domain: &str) -> Self {
        let labels = domain.split('.').map(|s| s.to_string()).collect();
        Self { labels }
    }

    fn to_bytes(&self, buf: &mut Vec<u8>) {
        for label in &self.labels {
            buf.push(label.len() as u8);
            buf.extend_from_slice(label.as_bytes());
        }

        buf.push(0); // terminator
    }
}

impl Question {
    fn create_query_question(domain: &str) -> Self {
        let qname = DomainName::from_str(domain);
        let qtype = 1; // Query A
        let qclass = 1; // Class IN (Internet)
        Question {
            qname,
            qtype,
            qclass,
        }
    }

    fn to_bytes(&self, buf: &mut Vec<u8>) {
        self.qname.to_bytes(buf);

        write_u16(buf, self.qtype);
        write_u16(buf, self.qclass);
    }
}

impl Message {
    fn create_query(domain: &str) -> Self {
        Self {
            header: Header::create_query_header(),
            question: vec![Question::create_query_question(domain)],
            answer: Vec::new(),
            authority: Vec::new(),
            additional: Vec::new(),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(512);

        self.header.to_bytes(&mut buf);

        // Questions
        for q in &self.question {
            q.to_bytes(&mut buf);
        }

        buf
    }
}

fn main() -> std::io::Result<()> {
    let query = Message::create_query("tomase.pt");
    println!("{:#?}", query);

    let packet = query.to_bytes();

    let socket = UdpSocket::bind("0.0.0.0:0")?;

    socket.send_to(&packet, "1.1.1.1:53")?;

    let mut buf = [0u8; 512];
    let (len, src) = socket.recv_from(&mut buf)?;

    println!("{:?}, {}, {}", buf, len, src);

    Ok(())
}
