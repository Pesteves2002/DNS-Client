use core::fmt;
use std::{collections::HashMap, error::Error, str::FromStr};

use crate::structs::{RefNode, read_label};

use super::write_u16;

use bytes::Buf;

#[derive(Debug)]
pub struct ParseQTypeError {
    pub value: String,
}

impl fmt::Display for ParseQTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid qtype: {}", self.value)
    }
}

impl std::error::Error for ParseQTypeError {}

#[derive(Debug)]
pub struct ParseQClassError {
    pub value: String,
}

impl fmt::Display for ParseQClassError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid qclass: {}", self.value)
    }
}

impl std::error::Error for ParseQClassError {}

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

impl FromStr for QType {
    type Err = ParseQTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(Self::A),
            "NS" => Ok(Self::NS),
            "CNAME" => Ok(Self::CNAME),
            "MX" => Ok(Self::MX),
            "TXT" => Ok(Self::TXT),
            "AAAA" => Ok(Self::AAAA),
            _ => Err(ParseQTypeError {
                value: s.to_string(),
            }),
        }
    }
}

impl QType {
    pub fn from_u16(value: u16) -> Result<Self, ParseQTypeError> {
        match value {
            1 => Ok(Self::A),
            2 => Ok(Self::NS),
            5 => Ok(Self::CNAME),
            15 => Ok(Self::MX),
            16 => Ok(Self::TXT),
            28 => Ok(Self::AAAA),
            _ => Err(ParseQTypeError {
                value: value.to_string(),
            }),
        }
    }

    pub fn to_bytes(self, buf: &mut Vec<u8>) {
        write_u16(buf, self as u16);
    }
}

impl FromStr for QClass {
    type Err = ParseQClassError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "IN" => Ok(Self::IN),
            "CS" => Ok(Self::CS),
            "CH" => Ok(Self::CH),
            "HS" => Ok(Self::HS),
            "ANY" => Ok(Self::ANY),
            _ => Err(ParseQClassError {
                value: s.to_string(),
            }),
        }
    }
}

impl QClass {
    pub fn from_u16(value: u16) -> Result<Self, ParseQClassError> {
        match value {
            1 => Ok(Self::IN),
            2 => Ok(Self::CS),
            3 => Ok(Self::CH),
            4 => Ok(Self::HS),
            255 => Ok(Self::ANY),
            _ => Err(ParseQClassError {
                value: value.to_string(),
            }),
        }
    }

    pub fn to_bytes(self, buf: &mut Vec<u8>) {
        write_u16(buf, self as u16);
    }
}

impl Question {
    pub fn create_query_question(
        domain: &str,
        typ: &str,
        class: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let qname = domain.trim().to_string();
        let qtype = QType::from_str(typ)?;
        let qclass = QClass::from_str(class)?;

        Ok(Question {
            qname,
            qtype,
            qclass,
        })
    }

    // Each label is written as follows:
    // [ LEN ][         CHARS         ]
    //
    // When finished terminate with a single 0:
    // [  0  ]
    //
    // e.g.
    // [  6  ]['t' 'o' 'm' 'a' 's' 'e']
    // [  0  ]
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
        nodes: &mut HashMap<usize, RefNode>,
    ) -> Result<Self, Box<dyn Error>> {
        let index = len - buf.remaining();

        let qname = read_label(buf, index, nodes)?;
        let qtype = QType::from_u16(buf.get_u16())?;
        let qclass = QClass::from_u16(buf.get_u16())?;

        Ok(Self {
            qname,
            qtype,
            qclass,
        })
    }
}

impl fmt::Display for Question {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "QNAME: {}", self.qname)?;
        writeln!(f, "QTYPE: {:?}", self.qtype)?;
        writeln!(f, "QCLASS: {:?}", self.qclass)
    }
}
