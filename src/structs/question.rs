use core::fmt;
use std::collections::HashMap;

use super::write_u16;

use bytes::Buf;

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

pub struct Question {
    qname: String,
    qtype: QType,
    qclass: QClass,
}

impl QType {
    pub fn from_str(s: &str) -> Option<Self> {
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

    pub fn from_u16(value: u16) -> Option<Self> {
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

    pub fn to_bytes(self, buf: &mut Vec<u8>) {
        write_u16(buf, self as u16);
    }
}

impl QClass {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "IN" => Some(Self::IN),
            "CH" => Some(Self::CH),
            "HS" => Some(Self::HS),
            "NONE" => Some(Self::NONE),
            "ANY" => Some(Self::ANY),
            _ => None,
        }
    }

    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(Self::IN),
            3 => Some(Self::CH),
            4 => Some(Self::HS),
            254 => Some(Self::NONE),
            255 => Some(Self::ANY),
            _ => None,
        }
    }

    pub fn to_bytes(self, buf: &mut Vec<u8>) {
        write_u16(buf, self as u16);
    }
}

impl Question {
    pub fn create_query_question(domain: &str, typ: &str, class: &str) -> Self {
        let qname = domain.to_string();
        let qtype = QType::from_str(typ).unwrap();
        let qclass = QClass::from_str(class).unwrap();

        Question {
            qname,
            qtype,
            qclass,
        }
    }

    pub fn to_bytes(&self, buf: &mut Vec<u8>) {
        for label in self.qname.split('.') {
            buf.push(label.len() as u8);
            buf.extend_from_slice(label.as_bytes());
        }

        buf.push(0); // terminator

        self.qtype.to_bytes(buf);
        self.qclass.to_bytes(buf);
    }

    pub fn from_bytes(buf: &mut &[u8], len: usize, labels: &mut HashMap<usize, String>) -> Self {
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
