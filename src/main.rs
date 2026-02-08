use bytes::Buf;
use std::{env, io::Error, io::ErrorKind, net::UdpSocket, vec};

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
    ttl: u32,
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

        let mut flags = 0;

        flags |= 1 << 8; // RD (recursion desired)

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

    fn from_bytes(buf: &mut &[u8]) -> Self {
        let id = buf.get_u16();
        let flags = buf.get_u16();
        let qdcount = buf.get_u16();
        let ancount = buf.get_u16();
        let nscount = buf.get_u16();
        let arcount = buf.get_u16();

        Header {
            id,
            flags,
            qdcount,
            ancount,
            nscount,
            arcount,
        }
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

    fn from_bytes(buf: &mut &[u8]) -> Self {
        let mut name = String::new();

        loop {
            let read = buf.get_u8();

            if read == 0 {
                break;
            }

            for _ in 0..read {
                let c = buf.get_u8();

                name.push(c as char);
            }

            name.push('.');
        }

        let qtype = buf.get_u16();

        let qclass = buf.get_u16();

        Self {
            qname: DomainName::from_str(&name),
            qtype,
            qclass,
        }
    }
}

impl Answer {
    fn from_bytes(buf: &mut &[u8]) -> Self {
        let mut name = String::new();

        loop {
            let read = buf.get_u8();

            let comp_bits = read >> 6 & 0b11;
            if comp_bits != 0 {
                let offset = read & 0x3F;
                let read = buf.get_u8();
                break;
            }

            let len = read & 0x3F;
            if len == 0 {
                break;
            }

            for _ in 0..len {
                let c = buf.get_u8();

                name.push(c as char);
            }

            name.push('.');
        }

        let rtype = buf.get_u16();
        let class = buf.get_u16();
        let ttl = buf.get_u32();
        let rdlength = buf.get_u16();

        let mut address = Vec::new();

        for _ in 0..rdlength {
            address.push(buf.get_u8());
        }

        println!("{:?}", address);

        let rdata = vec![];

        Self {
            name: DomainName::from_str(&name),
            rtype,
            class,
            ttl,
            rdlength,
            rdata,
        }
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

    fn from_bytes(buf: &mut &[u8]) -> Self {
        let header = Header::from_bytes(buf);

        let mut question = vec![];
        for _ in 0..header.qdcount {
            question.push(Question::from_bytes(buf));
        }

        let mut answer = vec![];
        for _ in 0..header.ancount {
            answer.push(Answer::from_bytes(buf));
        }

        Message {
            header,
            question,
            answer,
            authority: vec![],
            additional: vec![],
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(Error::new(ErrorKind::Other, "Not enough arguments"));
    }

    let packet = Message::create_query(args.get(1).unwrap()).to_bytes();

    let socket = UdpSocket::bind("0.0.0.0:0")?;

    socket.send_to(&packet, "8.8.8.8:53")?;

    let mut buf = [0u8; 512];
    let (len, _) = socket.recv_from(&mut buf)?;

    let mut data = &buf[..len];

    Message::from_bytes(&mut data);

    Ok(())
}
