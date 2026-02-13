use core::fmt;
use std::{collections::HashMap, error::Error};

use crate::structs::{answer::Answer, header::Header, question::Question};

pub struct Message {
    pub header: Header, // Always present
    question: Vec<Question>,
    answer: Vec<Answer>,
    authority: Vec<Answer>,
    additional: Vec<Answer>,
}

impl Message {
    pub fn create_query(domain: &str, qtype: &str, qclass: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            header: Header::create_query_header(),
            question: vec![Question::create_query_question(domain, qtype, qclass)?],
            answer: Vec::new(),
            authority: Vec::new(),
            additional: Vec::new(),
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(512); // Max UDP Message size

        self.header.to_bytes(&mut buf);

        for q in &self.question {
            q.to_bytes(&mut buf);
        }

        buf
    }

    pub fn from_bytes(buf: &[u8], len: usize) -> Result<Self, Box<dyn Error>> {
        let mut pointer = &buf[..len];
        let mut nodes = HashMap::new();

        let header = Header::from_bytes(&mut pointer);

        let question = (0..header.qdcount)
            .map(|_| Question::from_bytes(&mut pointer, len, &mut nodes))
            .collect::<Result<Vec<_>, _>>()?;

        let answer = (0..header.ancount)
            .map(|_| Answer::from_bytes(&mut pointer, len, &mut nodes))
            .collect::<Result<Vec<_>, _>>()?;

        let authority = (0..header.nscount)
            .map(|_| Answer::from_bytes(&mut pointer, len, &mut nodes))
            .collect::<Result<Vec<_>, _>>()?;

        let additional = (0..header.arcount)
            .map(|_| Answer::from_bytes(&mut pointer, len, &mut nodes))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Message {
            header,
            question,
            answer,
            authority,
            additional,
        })
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

#[cfg(test)]
mod tests {
    use dns_parser::QueryType;

    use super::*;

    #[test]
    fn create_query() {
        let domain = "tomase.pt";
        let qtype = "A";
        let qclass = "IN";

        let packet = Message::create_query(domain, qtype, qclass).unwrap();

        let mut builder = dns_parser::Builder::new_query(packet.header.id, true);

        builder.add_question(domain, false, QueryType::A, dns_parser::QueryClass::IN);

        let reference = builder.build().unwrap();

        assert_eq!(packet.to_bytes(), reference)
    }
}
