use core::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use bytes::Buf;

pub mod message;

mod answer;
mod header;
mod question;

#[derive(Debug)]
pub struct ParseLabelError {
    pub value: String,
}

impl fmt::Display for ParseLabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid label: {}", self.value)
    }
}

impl std::error::Error for ParseLabelError {}

fn write_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_be_bytes());
}

type RefNode = Rc<RefCell<Node>>;

struct Node {
    label: String,
    next: Option<RefNode>,
}

impl Node {
    fn new(label: String) -> RefNode {
        Rc::new(RefCell::new(Node { label, next: None }))
    }

    fn get_full_label(&self) -> String {
        let mut l = self.label.clone();
        l.push('.');

        if self.next.is_none() {
            return l.to_string();
        }

        l + &self.next.as_ref().unwrap().borrow().get_full_label()
    }
}

fn read_label(
    buf: &mut &[u8],
    mut index: usize,
    nodes: &mut HashMap<usize, RefNode>,
) -> Result<String, ParseLabelError> {
    let mut head: Option<RefNode> = None;
    let mut prev: Option<RefNode> = None;

    loop {
        let octet = buf.get_u8();

        let comp_bits = octet & 0xC0; // (first 2 bits)
        if comp_bits == 0xC0 {
            let upper = octet & 0x3F;
            let lower = buf.get_u8();

            let offset = ((upper as u16) << 8 | lower as u16) as usize;
            match nodes.get(&offset) {
                Some(node) => {
                    if let Some(p) = prev {
                        p.borrow_mut().next = Some(node.clone());
                    }

                    if head.is_none() {
                        head = Some(node.clone());
                    }
                }

                None => {
                    return Err(ParseLabelError {
                        value: "No entry on nodes".to_string(),
                    });
                }
            }

            break;
        }

        let len = octet & 0x3F; // (last 6 bits)
        // Terminate with 0
        if len == 0 {
            break;
        }

        let label = (0..len).map(|_| buf.get_u8() as char).collect();

        let node = Node::new(label);

        nodes.insert(index, node.clone());

        if head.is_none() {
            head = Some(node.clone());
        }

        if let Some(p) = prev {
            p.borrow_mut().next = Some(node.clone());
        }

        prev = Some(node.clone());

        // Include first octet
        index += 1 + len as usize
    }

    if head.is_none() {
        return Err(ParseLabelError {
            value: "No head detected".to_string(),
        });
    }

    Ok(head.unwrap().borrow().get_full_label())
}
