use std::collections::{HashMap, HashSet};
use std::io::StdoutLock;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use rustrom::main_loop;
use rustrom::node::{Event, Node};
use rustrom::protocol::{Body, Init, Message};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Payload {
    Gossip {
        seen: HashSet<u32>,
    },
    Broadcast {
        message: u32,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: HashSet<u32>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

enum InjectedPayload {
    Gossip,
}

#[derive(Debug)]
struct BroadcastNode {
    id: String,
    msg_id: u32,
    messages: HashSet<u32>,
    neighbors: Vec<String>,
    known: HashMap<String, HashSet<u32>>,
}

impl Node<Payload, InjectedPayload> for BroadcastNode {
    fn from_init(init: Init, tx: Sender<Event<Payload, InjectedPayload>>) -> Self {
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(150));
            tx.send(Event::Injected(InjectedPayload::Gossip))
                .expect("failed to send gossip event");
        });

        Self {
            id: init.node_id,
            msg_id: 1,
            messages: Default::default(),
            neighbors: Default::default(),
            known: init
                .node_ids
                .into_iter()
                .map(|nid| (nid, HashSet::new()))
                .collect(),
        }
    }

    fn step(
        &mut self,
        input: Event<Payload, InjectedPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match input {
            Event::EOF => {}
            Event::Injected(payload) => match payload {
                InjectedPayload::Gossip => {
                    for n in &self.neighbors {
                        let known_to_n = &self.known[n];
                        let need_to_known = self
                            .messages
                            .iter()
                            .filter(|m| !known_to_n.contains(m))
                            .copied()
                            .collect();

                        Message {
                            src: self.id.clone(),
                            dst: n.clone(),
                            body: Body {
                                id: None,
                                in_reply_to: None,
                                payload: Payload::Gossip {
                                    seen: need_to_known,
                                },
                            },
                        }
                        .send(output)?;
                    }
                }
            },
            Event::Message(input) => {
                let payload = match input.body.payload {
                    Payload::Gossip { ref seen } => {
                        if let Some(known) = self.known.get_mut(&input.dst) {
                            known.extend(seen.iter().copied());
                        }
                        self.messages.extend(seen);
                        None
                    }
                    Payload::Broadcast { message } => {
                        self.messages.insert(message);
                        Some(Payload::BroadcastOk)
                    }
                    Payload::Read => Some(Payload::ReadOk {
                        messages: self.messages.clone(),
                    }),
                    Payload::Topology { ref topology } => {
                        if let Some(neighbors) = topology.get(&self.id) {
                            self.neighbors = neighbors.clone();
                        }
                        Some(Payload::TopologyOk)
                    }
                    Payload::BroadcastOk | Payload::ReadOk { .. } | Payload::TopologyOk => None,
                };
                if let Some(payload) = payload {
                    let reply = input.into_reply_with_payload(Some(&mut self.msg_id), payload);
                    reply.send(output)?;
                }
            }
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<BroadcastNode, _, _>()
}
