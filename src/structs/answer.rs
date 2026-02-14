use core::fmt;
use std::{collections::HashMap, error::Error};

use bytes::Buf;

use crate::structs::{
    RefNode,
    question::{QClass, QType},
    read_label,
};

#[derive(Debug, Clone)]
struct DomainName(String);

impl From<String> for DomainName {
    fn from(s: String) -> Self {
        DomainName(s)
    }
}

impl fmt::Display for DomainName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
enum RDATA {
    DomainName(DomainName), // CNAME, NS, PTR
    IPV4([u8; 4]),          // A
    IPV6([u8; 16]),         // AAAA
    TXT(String),            // TXT
    MX(u16, String),        // MX
    SOA(DomainName, DomainName, u32, u32, u32, u32, u32),
}

impl fmt::Display for RDATA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DomainName(s) => {
                writeln!(f, "{s}")?;
            }

            Self::TXT(s) => {
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

            Self::SOA(mname, rname, serial, refresh, retry, expire, minimum) => {
                writeln!(f)?;
                writeln!(f, "MNAME: {mname}")?;
                writeln!(f, "RNAME: {rname}")?;
                writeln!(f, "SERIAL: {serial}")?;
                writeln!(f, "REFRESH: {refresh}")?;
                writeln!(f, "RETRY: {retry}")?;
                writeln!(f, "EXPIRE: {expire}")?;
                writeln!(f, "MINIMUM: {minimum}")?;
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
    pub fn from_bytes(
        buf: &mut &[u8],
        len: usize,
        nodes: &mut HashMap<usize, RefNode>,
    ) -> Result<Self, Box<dyn Error>> {
        let index = len - buf.remaining();

        let qname = read_label(buf, index, nodes)?;
        let rtype = QType::from_u16(buf.get_u16())?;

        let class = QClass::from_u16(buf.get_u16())?;
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
                let name = read_label(buf, index, nodes)?;

                RDATA::DomainName(DomainName::from(name))
            }

            QType::MX => {
                let pref = buf.get_u16();

                let index = len - buf.remaining();
                let name = read_label(buf, index, nodes)?;

                RDATA::MX(pref, name)
            }

            QType::SOA => {
                let index = len - buf.remaining();
                let mname = read_label(buf, index, nodes)?;

                let index = len - buf.remaining();
                let rname = read_label(buf, index, nodes)?;

                let serial = buf.get_u32();
                let refresh = buf.get_u32();
                let retry = buf.get_u32();
                let expire = buf.get_u32();
                let minimum = buf.get_u32();

                RDATA::SOA(
                    DomainName::from(mname),
                    DomainName::from(rname),
                    serial,
                    refresh,
                    retry,
                    expire,
                    minimum,
                )
            }
        };

        Ok(Self {
            name: qname,
            rtype,
            class,
            ttl,
            rdlength,
            rdata,
        })
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
