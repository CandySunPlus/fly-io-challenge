use std::collections::HashMap;
use std::io::StdoutLock;

use rustrom::main_loop;
use rustrom::node::Node;
use rustrom::protocol::{Init, Message};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Payload {
    Broadcast {
        message: u32,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<u32>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

#[allow(dead_code)]
#[derive(Debug)]
struct BroadcastNode {
    id: String,
    msg_id: u32,
    messages: Vec<u32>,
    neighbors: Vec<String>,
}

impl Node<Payload> for BroadcastNode {
    fn from_init(init: Init) -> Self {
        BroadcastNode {
            id: init.node_id,
            msg_id: 1,
            messages: vec![],
            neighbors: vec![],
        }
    }

    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let payload = match input.body.payload {
            Payload::Broadcast { message: ref msg } => {
                if !self.messages.contains(msg) {
                    self.messages.push(*msg);
                    for neighbor in &self.neighbors {
                        // Don't send the message back to the sender
                        if input.src == neighbor.as_str() {
                            continue;
                        }
                        let mut broadcast_msg = input.clone();
                        broadcast_msg.dst = neighbor.clone();
                        broadcast_msg.src = self.id.clone();
                        broadcast_msg.body.id = None;
                        broadcast_msg.body.in_reply_to = None;
                        broadcast_msg.send(output)?;
                    }
                }
                if input.body.id.is_some() {
                    Payload::BroadcastOk
                } else {
                    return Ok(());
                }
            }
            Payload::BroadcastOk => unreachable!(),
            Payload::Read => Payload::ReadOk {
                messages: self.messages.clone(),
            },
            Payload::ReadOk { .. } => unreachable!(),
            Payload::Topology { ref topology } => {
                if let Some(neighbors) = topology.get(&self.id) {
                    self.neighbors = neighbors.clone();
                }
                Payload::TopologyOk
            }
            Payload::TopologyOk => unreachable!(),
        };
        let reply = input.into_reply_with_payload(Some(&mut self.msg_id), payload);
        reply.send(output)
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<BroadcastNode, _>()
}
