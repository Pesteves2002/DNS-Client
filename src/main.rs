use bytes::Buf;
use std::{collections::HashMap, env, io::Error, net::UdpSocket, vec};

use rand::Rng;

fn write_u16(buf: &mut Vec<u8>, v: u16) {
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

#[derive(Debug, Clone)]
struct DomainName {
    labels: Vec<String>,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum QType {
    A = 1,
    NS = 2,
    CNAME = 5,
    SOA = 6,
    PTR = 12,
    MX = 15,
    TXT = 16,
    AAAA = 28,
    SRV = 33,
    OPT = 41,
    CAA = 257,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum QClass {
    IN = 1,
    CH = 3,
    HS = 4,
    NONE = 254,
    ANY = 255,
}

#[derive(Debug)]
struct Question {
    qname: DomainName,
    qtype: QType,
    qclass: QClass,
}

#[derive(Debug)]
struct Answer {
    name: DomainName,
    rtype: QType,
    class: QClass,
    ttl: u32,
    rdlength: u16,
    rdata: Vec<u8>,
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

impl QType {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "A" => Some(Self::A),
            "NS" => Some(Self::NS),
            "CNAME" => Some(Self::CNAME),
            "SOA" => Some(Self::SOA),
            "PTR" => Some(Self::PTR),
            "MX" => Some(Self::MX),
            "TXT" => Some(Self::TXT),
            "AAAA" => Some(Self::AAAA),
            "SRV" => Some(Self::SRV),
            "OPT" => Some(Self::OPT),
            "CAA" => Some(Self::CAA),
            _ => None,
        }
    }

    fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(Self::A),
            2 => Some(Self::NS),
            5 => Some(Self::CNAME),
            6 => Some(Self::SOA),
            12 => Some(Self::PTR),
            15 => Some(Self::MX),
            16 => Some(Self::TXT),
            28 => Some(Self::AAAA),
            33 => Some(Self::SRV),
            41 => Some(Self::OPT),
            257 => Some(Self::CAA),
            _ => None,
        }
    }

    fn to_bytes(self, buf: &mut Vec<u8>) {
        write_u16(buf, self as u16);
    }
}

impl QClass {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "IN" => Some(Self::IN),
            "CH" => Some(Self::CH),
            "HS" => Some(Self::HS),
            "NONE" => Some(Self::NONE),
            "ANY" => Some(Self::ANY),
            _ => None,
        }
    }

    fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(Self::IN),
            3 => Some(Self::CH),
            4 => Some(Self::HS),
            254 => Some(Self::NONE),
            255 => Some(Self::ANY),
            _ => None,
        }
    }

    fn to_bytes(self, buf: &mut Vec<u8>) {
        write_u16(buf, self as u16);
    }
}

impl Question {
    fn create_query_question(domain: &str, typ: &str, class: &str) -> Self {
        let qname = DomainName::from_str(domain);
        let qtype = QType::from_str(typ).unwrap();
        let qclass = QClass::from_str(class).unwrap();

        Question {
            qname,
            qtype,
            qclass,
        }
    }

    fn to_bytes(&self, buf: &mut Vec<u8>) {
        self.qname.to_bytes(buf);

        self.qtype.to_bytes(buf);
        self.qclass.to_bytes(buf);
    }

    fn from_bytes(buf: &mut &[u8], len: usize, labels: &mut HashMap<usize, DomainName>) -> Self {
        let mut name = String::new();

        let index = len - buf.remaining();

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

        let domain_name = DomainName::from_str(&name);
        labels.insert(index, domain_name.clone());

        let qtype = buf.get_u16();

        let qclass = buf.get_u16();

        Self {
            qname: domain_name,
            qtype: QType::from_u16(qtype).unwrap(),
            qclass: QClass::from_u16(qclass).unwrap(),
        }
    }
}

impl Answer {
    fn from_bytes(buf: &mut &[u8], len: usize, labels: &mut HashMap<usize, DomainName>) -> Self {
        let mut name = String::new();
        let mut domain_name = None;

        let idx = len - buf.len();

        loop {
            let read = buf.get_u8();

            let comp_bits = read >> 6 & 0b11;
            if comp_bits != 0 {
                let offset = read & 0x3F;
                let read = buf.get_u8();

                let full_offset = ((offset as u16) << 8 | read as u16) as usize;
                if let Some(label) = labels.get(&full_offset) {
                    domain_name = Some(label.clone());
                }

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

        if domain_name.is_none() {
            let domain = DomainName::from_str(&name);
            labels.insert(idx, domain.clone());
            domain_name = Some(domain);
        }

        let rtype = buf.get_u16();
        let class = buf.get_u16();
        let ttl = buf.get_u32();
        let rdlength = buf.get_u16();

        let mut rdata = Vec::new();

        for _ in 0..rdlength {
            rdata.push(buf.get_u8());
        }

        Self {
            name: domain_name.unwrap(),
            rtype: QType::from_u16(rtype).unwrap(),
            class: QClass::from_u16(class).unwrap(),
            ttl,
            rdlength,
            rdata,
        }
    }
}

impl Message {
    fn create_query(domain: &str, qtype: &str, qclass: &str) -> Self {
        Self {
            header: Header::create_query_header(),
            question: vec![Question::create_query_question(domain, qtype, qclass)],
            answer: Vec::new(),
            authority: Vec::new(),
            additional: Vec::new(),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(512);

        self.header.to_bytes(&mut buf);

        for q in &self.question {
            q.to_bytes(&mut buf);
        }

        buf
    }

    fn from_bytes(buf: &mut &[u8]) -> Self {
        let len = buf.remaining();

        let header = Header::from_bytes(buf);

        let mut labels: HashMap<usize, DomainName> = HashMap::new();

        let mut question = vec![];
        for _ in 0..header.qdcount {
            question.push(Question::from_bytes(buf, len, &mut labels));
        }

        let mut answer = vec![];
        for _ in 0..header.ancount {
            answer.push(Answer::from_bytes(buf, len, &mut labels));
        }

        let mut authority = vec![];
        for _ in 0..header.nscount {
            authority.push(Answer::from_bytes(buf, len, &mut labels));
        }

        let mut additional = vec![];
        for _ in 0..header.nscount {
            additional.push(Answer::from_bytes(buf, len, &mut labels));
        }

        Message {
            header,
            question,
            answer,
            authority,
            additional,
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        return Err(Error::other("Not enough arguments"));
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
    println!("{:#?}", response);

    Ok(())
}
