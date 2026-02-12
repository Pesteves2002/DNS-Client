use core::fmt;
use std::collections::HashMap;

use bytes::Buf;

use crate::structs::{answer::Answer, header::Header, question::Question};

pub struct Message {
    pub header: Header, // Always present
    question: Vec<Question>,
    answer: Vec<Answer>,
    authority: Vec<Answer>,
    additional: Vec<Answer>,
}

impl Message {
    pub fn create_query(domain: &str, qtype: &str, qclass: &str) -> Self {
        Self {
            header: Header::create_query_header(),
            question: vec![Question::create_query_question(domain, qtype, qclass)],
            answer: Vec::new(),
            authority: Vec::new(),
            additional: Vec::new(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(512); // Max UDP Message size

        self.header.to_bytes(&mut buf);

        for q in &self.question {
            q.to_bytes(&mut buf);
        }

        buf
    }

    pub fn from_bytes(buf: &mut &[u8]) -> Self {
        let len = buf.remaining();

        let header = Header::from_bytes(buf);

        let mut labels: HashMap<usize, String> = HashMap::new();

        let mut question = vec![];
        for _ in 0..header.qdcount {
            question.push(Question::from_bytes(buf, len, &mut labels));
        }

        let mut answer = vec![];
        for _ in 0..header.ancount {
            answer.push(Answer::from_bytes(buf, len, &mut labels));
        }

        let mut authority = vec![];
        for _ in 0..header.nscount {
            authority.push(Answer::from_bytes(buf, len, &mut labels));
        }

        let mut additional = vec![];
        for _ in 0..header.nscount {
            additional.push(Answer::from_bytes(buf, len, &mut labels));
        }

        Message {
            header,
            question,
            answer,
            authority,
            additional,
        }
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

        let packet = Message::create_query(domain, qtype, qclass);

        let mut builder = dns_parser::Builder::new_query(packet.header.id, true);

        builder.add_question(domain, false, QueryType::A, dns_parser::QueryClass::IN);

        let reference = builder.build().unwrap();

        assert_eq!(packet.to_bytes(), reference)
    }
}
