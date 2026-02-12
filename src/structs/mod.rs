use std::collections::HashMap;

use bytes::Buf;

pub mod message;

mod answer;
mod header;
mod question;

fn write_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_be_bytes());
}

fn read_label(
    buf: &mut &[u8],
    mut index: usize,
    labels: &mut HashMap<usize, String>,
) -> Option<String> {
    let mut qname = String::new();
    loop {
        let read = buf.get_u8();

        let comp_bits = read >> 6 & 0b11;
        if comp_bits == 0b11 {
            let offset = read & 0x3F;
            let read = buf.get_u8();

            let mut full_offset = ((offset as u16) << 8 | read as u16) as usize;
            while let Some(label) = labels.get(&full_offset) {
                qname.push_str(label);
                qname.push('.');

                full_offset += 1 + label.len();
            }

            break;
        }

        let len = read & 0x3F;
        // Terminate with 0
        if len == 0 {
            break;
        }

        let mut part = String::with_capacity((read + 1) as usize);

        for _ in 0..read {
            part.push(buf.get_u8() as char);
        }

        labels.insert(index, part.clone());

        part.push('.');
        qname.push_str(&part);

        index += 1 + read as usize
    }

    Some(qname)
}
