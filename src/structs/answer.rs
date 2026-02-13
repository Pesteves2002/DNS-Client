use core::fmt;
use std::collections::HashMap;

use bytes::Buf;

use crate::structs::{
    RefNode,
    question::{QClass, QType},
    read_label,
};

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
enum RDATA {
    DomainName(String), // CNAME, NS, PTR
    IPV4([u8; 4]),      // A
    IPV6([u8; 16]),     // AAAA
    TXT(String),        // TXT
    MX(u16, String),    // MX
}

impl fmt::Display for RDATA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DomainName(s) | Self::TXT(s) => {
                writeln!(f, "{s}")?;
            }

            Self::MX(pref, s) => {
                writeln!(f, "{s} ({pref})")?;
            }

            Self::IPV4(ip) => {
                writeln!(f, "{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])?;
            }

            Self::IPV6(ip) => {
                for i in 0..8 {
                    if i != 0 {
                        write!(f, ":")?;
                    }
                    let segment = ((ip[2 * i] as u16) << 8) | ip[2 * i + 1] as u16;
                    write!(f, "{:x}", segment)?;
                }
            }
        };

        Ok(())
    }
}

pub struct Answer {
    name: String,
    rtype: QType,
    class: QClass,
    ttl: u32,
    rdlength: u16,
    rdata: RDATA,
}

impl Answer {
    pub fn from_bytes(buf: &mut &[u8], len: usize, nodes: &mut HashMap<usize, RefNode>) -> Self {
        let index = len - buf.remaining();

        let qname = read_label(buf, index, nodes).unwrap();
        let rtype = buf.get_u16();
        let rtype = QType::from_u16(rtype).unwrap();

        let class = buf.get_u16();
        let ttl = buf.get_u32();
        let rdlength = buf.get_u16();

        let rdata = match rtype {
            QType::A => {
                let mut v = Vec::new();
                for _ in 0..rdlength {
                    v.push(buf.get_u8());
                }

                assert_eq!(v.len(), 4, "Invalid A record length");
                RDATA::IPV4([v[0], v[1], v[2], v[3]])
            }

            QType::AAAA => {
                let mut v = Vec::new();
                for _ in 0..rdlength {
                    v.push(buf.get_u8());
                }

                assert_eq!(v.len(), 16, "Invalid AAAA record length");
                let mut addr = [0u8; 16];
                addr.copy_from_slice(&v);
                RDATA::IPV6(addr)
            }

            QType::TXT => {
                let mut v = Vec::new();

                let len = buf.get_u8();
                for _ in 0..len {
                    let c = buf.get_u8();
                    v.push(c);
                }

                let s: String = v.iter().map(|&b| b as char).collect();
                RDATA::TXT(s)
            }

            QType::CNAME | QType::NS => {
                let index = len - buf.remaining();
                let name = read_label(buf, index, nodes).unwrap();

                RDATA::DomainName(name)
            }

            QType::MX => {
                let pref = buf.get_u16();

                let index = len - buf.remaining();
                let name = read_label(buf, index, nodes).unwrap();

                RDATA::MX(pref, name)
            }
        };

        Self {
            name: qname,
            rtype,
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
        writeln!(f, "RDATA: {}", self.rdata)?;

        Ok(())
    }
}
