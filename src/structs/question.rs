use core::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::structs::{Node, read_label};

use super::write_u16;

use bytes::Buf;

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum QType {
    A = 1,
    NS = 2,
    CNAME = 5,
    MX = 15,
    TXT = 16,
    AAAA = 28,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum QClass {
    IN = 1,
    CS = 2,
    CH = 3,
    HS = 4,
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
            "MX" => Some(Self::MX),
            "TXT" => Some(Self::TXT),
            "AAAA" => Some(Self::AAAA),
            _ => None,
        }
    }

    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(Self::A),
            2 => Some(Self::NS),
            5 => Some(Self::CNAME),
            15 => Some(Self::MX),
            16 => Some(Self::TXT),
            28 => Some(Self::AAAA),
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
            "CS" => Some(Self::CS),
            "CH" => Some(Self::CH),
            "HS" => Some(Self::HS),
            "ANY" => Some(Self::ANY),
            _ => None,
        }
    }

    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(Self::IN),
            2 => Some(Self::CS),
            3 => Some(Self::CH),
            4 => Some(Self::HS),
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
        let qname = domain.trim().to_string();
        let qtype = QType::from_str(typ).unwrap();
        let qclass = QClass::from_str(class).unwrap();

        Question {
            qname,
            qtype,
            qclass,
        }
    }

    // Each label is written as follows:
    // [ LEN ][         CHARS         ]
    //
    // When finished terminate with a single 0:
    // [0]
    //
    // e.g.
    // [  6  ]['t' 'o' 'm' 'a' 's' 'e']
    pub fn to_bytes(&self, buf: &mut Vec<u8>) {
        for label in self.qname.split('.') {
            buf.push(label.len() as u8);
            buf.extend_from_slice(label.as_bytes());
        }

        buf.push(0); // terminator

        self.qtype.to_bytes(buf);
        self.qclass.to_bytes(buf);
    }

    pub fn from_bytes(
        buf: &mut &[u8],
        len: usize,
        nodes: &mut HashMap<usize, Rc<RefCell<Node>>>,
    ) -> Self {
        let index = len - buf.remaining();

        let qname = read_label(buf, index, nodes).unwrap();
        let qtype = buf.get_u16();
        let qclass = buf.get_u16();

        Self {
            qname,
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
