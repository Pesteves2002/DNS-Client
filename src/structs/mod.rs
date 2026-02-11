fn write_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_be_bytes());
}

pub mod answer;
pub mod header;
pub mod message;
pub mod question;
