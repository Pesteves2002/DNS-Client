use core::fmt;

use super::write_u16;

use bytes::Buf;
use rand::Rng;

pub struct Header {
    pub id: u16,
    flags: u16,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

impl Header {
    pub fn create_query_header() -> Self {
        let id = rand::rng().random();

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

    pub fn to_bytes(&self, buf: &mut Vec<u8>) {
        write_u16(buf, self.id);
        write_u16(buf, self.flags);
        write_u16(buf, self.qdcount);
        write_u16(buf, self.ancount);
        write_u16(buf, self.nscount);
        write_u16(buf, self.arcount);
    }

    pub fn from_bytes(buf: &mut &[u8]) -> Self {
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
