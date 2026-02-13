use std::{cell::RefCell, collections::HashMap, rc::Rc};

use bytes::Buf;

pub mod message;

mod answer;
mod header;
mod question;

fn write_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_be_bytes());
}

struct Node {
    label: String,
    next: Option<Rc<RefCell<Node>>>,
}

impl Node {
    fn new(label: String) -> Rc<RefCell<Self>> {
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
    nodes: &mut HashMap<usize, Rc<RefCell<Node>>>,
) -> Option<String> {
    let mut prev: Option<Rc<RefCell<Node>>> = None;
    let mut head: Option<Rc<RefCell<Node>>> = None;

    loop {
        let read = buf.get_u8();

        let comp_bits = read >> 6 & 0b11;
        if comp_bits == 0b11 {
            let offset = read & 0x3F;
            let read = buf.get_u8();

            let full_offset = ((offset as u16) << 8 | read as u16) as usize;
            if let Some(node) = nodes.get(&full_offset) {
                if let Some(p) = prev {
                    p.borrow_mut().next = Some(node.clone());
                }

                if head.is_none() {
                    head = Some(node.clone());
                }
            }

            break;
        }

        let len = read & 0x3F;
        // Terminate with 0
        if len == 0 {
            break;
        }

        let label = (0..read).map(|_| buf.get_u8() as char).collect();

        let node = Node::new(label);

        nodes.insert(index, node.clone());

        if let Some(p) = prev {
            p.borrow_mut().next = Some(node.clone());
        }

        prev = Some(node.clone());

        if head.is_none() {
            head = Some(node.clone());
        }

        index += 1 + read as usize
    }

    head.as_ref()?;

    Some(head.unwrap().borrow().get_full_label())
}
