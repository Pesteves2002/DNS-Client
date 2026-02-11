use core::fmt;
use std::collections::HashMap;

use bytes::Buf;

use crate::structs::question::{QClass, QType};

pub struct Answer {
    name: String,
    rtype: QType,
    class: QClass,
    ttl: u32,
    rdlength: u16,
    rdata: Vec<u8>,
}

impl Answer {
    pub fn from_bytes(buf: &mut &[u8], len: usize, labels: &mut HashMap<usize, String>) -> Self {
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
