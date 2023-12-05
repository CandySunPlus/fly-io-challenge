use std::collections::HashMap;
use std::io::StdoutLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::Context;
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

type Callback = Box<dyn FnOnce()>;

#[derive(Debug, Clone)]
struct NeedReplyMessage {
    message: Arc<Mutex<Message<Payload>>>,
    sent: Arc<AtomicBool>,
}

impl NeedReplyMessage {
    fn new(message: Message<Payload>) -> Self {
        Self {
            message: Arc::new(Mutex::new(message)),
            sent: Default::default(),
        }
    }
}

#[allow(dead_code)]
struct BroadcastNode {
    id: String,
    msg_id: u32,
    messages: Vec<u32>,
    neighbors: Vec<String>,
    callbacks: HashMap<u32, Callback>,
    message_tx: Sender<NeedReplyMessage>,
}

impl Node<Payload> for BroadcastNode {
    fn from_init(init: Init) -> Self {
        let (tx, rx) = mpsc::channel::<NeedReplyMessage>();

        let tx_rc = tx.clone();
        thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                if msg.sent.load(Ordering::SeqCst) {
                    msg.message
                        .lock()
                        .unwrap()
                        .send(&mut std::io::stdout().lock())
                        .unwrap();

                    tx_rc.send(msg).unwrap();
                    thread::sleep(Duration::from_millis(300));
                }
            }
        });

        Self {
            id: init.node_id,
            msg_id: 1,
            messages: vec![],
            neighbors: vec![],
            callbacks: Default::default(),
            message_tx: tx,
        }
    }

    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let payload = match input.body.payload {
            Payload::Broadcast { message: ref msg } => {
                if !self.messages.contains(msg) {
                    self.messages.push(*msg);
                    eprintln!("get {msg} from {}", input.src);
                    for neighbor in &self.neighbors {
                        self.msg_id += 1;
                        // Don't send the message back to the sender again
                        if input.src == neighbor.as_str() {
                            continue;
                        }
                        let mut broadcast_msg = input.clone();
                        broadcast_msg.dst = neighbor.clone();
                        broadcast_msg.src = self.id.clone();
                        broadcast_msg.body.id = Some(self.msg_id);
                        broadcast_msg.body.in_reply_to = None;
                        broadcast_msg.send(output)?;

                        let need_reply_message = NeedReplyMessage::new(broadcast_msg);

                        let need_reply_message_rc = need_reply_message.clone();
                        self.callbacks.insert(
                            self.msg_id,
                            Box::new(move || {
                                need_reply_message_rc.sent.store(true, Ordering::SeqCst)
                            }),
                        );

                        self.message_tx
                            .send(need_reply_message)
                            .context("failed to send broadcast to channel")?;
                    }
                }
                if input.body.id.is_some() {
                    Payload::BroadcastOk
                } else {
                    return Ok(());
                }
            }
            Payload::Read => Payload::ReadOk {
                messages: self.messages.clone(),
            },
            Payload::Topology { ref topology } => {
                if let Some(neighbors) = topology.get(&self.id) {
                    self.neighbors = neighbors.clone();
                }
                Payload::TopologyOk
            }
            Payload::BroadcastOk => {
                if let Some(msg_id) = input.body.in_reply_to {
                    if let Some(callback) = self.callbacks.remove(&msg_id) {
                        eprintln!("callback for msg_id: {msg_id} from {}", input.src);
                        callback();
                    }
                }
                return Ok(());
            }
            Payload::ReadOk { .. } | Payload::TopologyOk => unreachable!(),
        };
        let reply = input.into_reply_with_payload(Some(&mut self.msg_id), payload);
        reply.send(output)
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<BroadcastNode, _>()
}
