use bytes::Buf;
use core::fmt;
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
    qname: String,
    qtype: QType,
    qclass: QClass,
}

#[derive(Debug)]
struct Answer {
    name: String,
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

impl fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ID: {}", self.id)?;

        writeln!(f, "QR: {}", self.flags >> 15)?;
        writeln!(f, "OPCODE: {}", self.flags >> 11 & 0x7)?;
        writeln!(f, "AA: {}", self.flags >> 10 & 0x1)?;
        writeln!(f, "TC: {}", self.flags >> 9 & 0x1)?;
        writeln!(f, "RD: {}", self.flags >> 8 & 0x1)?;
        writeln!(f, "RA: {}", self.flags >> 7 & 0x1)?;
        writeln!(f, "Z: {}", self.flags >> 4 & 0x7)?;
        writeln!(f, "RCODE: {}", self.flags & 0xF)?;

        writeln!(f, "QDCOUNT: {}", self.qdcount)?;
        writeln!(f, "ANCOUNT: {}", self.ancount)?;
        writeln!(f, "NSCOUNT: {}", self.nscount)?;
        writeln!(f, "ARCOUNT: {}", self.arcount)
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
        let qname = domain.to_string();
        let qtype = QType::from_str(typ).unwrap();
        let qclass = QClass::from_str(class).unwrap();

        Question {
            qname,
            qtype,
            qclass,
        }
    }

    fn to_bytes(&self, buf: &mut Vec<u8>) {
        for label in self.qname.split('.') {
            buf.push(label.len() as u8);
            buf.extend_from_slice(label.as_bytes());
        }

        buf.push(0); // terminator

        self.qtype.to_bytes(buf);
        self.qclass.to_bytes(buf);
    }

    fn from_bytes(buf: &mut &[u8], len: usize, labels: &mut HashMap<usize, String>) -> Self {
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

        let domain_name = name;
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

impl fmt::Display for Question {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "QNAME: {}", self.qname)?;
        writeln!(f, "QTYPE: {:?}", self.qtype)?;
        writeln!(f, "QCLASS: {:?}", self.qclass)
    }
}

impl Answer {
    fn from_bytes(buf: &mut &[u8], len: usize, labels: &mut HashMap<usize, String>) -> Self {
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
            labels.insert(idx, name.clone());
            domain_name = Some(name);
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

impl fmt::Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "NAME: {}", self.name)?;
        writeln!(f, "TYPE: {:?}", self.rtype)?;
        writeln!(f, "CLASS: {:?}", self.class)?;
        writeln!(f, "TTL: {}", self.ttl)?;
        writeln!(f, "RDLENGTH: {}", self.rdlength)?;
        writeln!(f, "RDATA: {:?}", self.rdata)?;

        Ok(())
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

        let mut labels: HashMap<usize, String> = HashMap::new();

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

impl fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "##### MESSAGE #####")?;

        writeln!(f, "### HEADER ###")?;
        writeln!(f, "{}", self.header)?;

        for q in &self.question {
            writeln!(f, "### QUESTION ###")?;
            writeln!(f, "{q}")?;
        }

        for a in &self.answer {
            writeln!(f, "### ANSWER ###")?;
            writeln!(f, "{a}")?;
        }

        for a in &self.authority {
            writeln!(f, "### AUTHORITY ###")?;
            writeln!(f, "{a}")?;
        }

        for a in &self.additional {
            writeln!(f, "### ADDITIONAL ###")?;
            writeln!(f, "{a}")?;
        }

        Ok(())
    }
}

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
