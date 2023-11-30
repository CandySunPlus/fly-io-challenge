use std::sync::atomic::{AtomicU32, Ordering};

use crate::protocol::{Message, MessageBody, MessageType};
use crate::Result;

#[derive(Debug, Default)]
pub struct Node {
    node_id: Option<String>,
    ids: Vec<u32>,
    neighbors: Vec<String>,
    msg_id: AtomicU32,
}

impl Node {
    pub async fn process(&mut self, msg: Message) -> Result<()> {
        match msg.r#type() {
            MessageType::Echo { echo } => self.reply(
                &msg,
                MessageType::EchoOk {
                    echo: echo.to_owned(),
                },
            ),
            MessageType::Init {
                node_id,
                node_ids: _,
            } => {
                self.node_id = Some(node_id.to_owned());
                self.reply(&msg, MessageType::InitOk {});
            }
            MessageType::Generate {} => self.reply(
                &msg,
                MessageType::GenerateOk {
                    id: uuid::Uuid::new_v4().to_string(),
                },
            ),
            MessageType::Broadcast { message } => {
                if !self.ids.contains(message) {
                    self.ids.push(message.to_owned());
                    for neighbor in self.neighbors.iter() {
                        // don't send back to the sender
                        if neighbor != msg.src() {
                            self.send(neighbor, msg.r#type().clone())?;
                        }
                    }
                }
                if !msg.id().is_none() {
                    self.reply(&msg, MessageType::BroadcastOk {});
                }
            }
            MessageType::Read {} => self.reply(
                &msg,
                MessageType::ReadOk {
                    messages: self.ids.clone(),
                },
            ),
            MessageType::Topology { topology } => {
                let Some(node_id) = self.node_id.as_ref() else {
                    return Err("node has not been initialized".into());
                };
                if let Some(neighbors) = topology.get(node_id) {
                    self.neighbors = neighbors.clone();
                }
                self.reply(&msg, MessageType::TopologyOk {})
            }
            _ => self.reply(
                &msg,
                MessageType::Error {
                    code: 10,
                    text: "not-supported".to_owned(),
                },
            ),
        }

        Ok(())
    }

    fn send(&self, dest: &str, msg_type: MessageType) -> Result<()> {
        let Some(node_id) = self.node_id.as_ref() else {
            return Err("node has not been initialized".into());
        };
        let message = Message::new(&node_id, dest, MessageBody::new(msg_type, None, None));
        println!(
            "{}",
            serde_json::to_string(&message).expect("json serialize error")
        );
        Ok(())
    }

    fn reply(&mut self, msg: &Message, msg_type: MessageType) {
        if let Some(reply_message) =
            msg.create_reply_msg(Some(self.msg_id.fetch_add(1, Ordering::SeqCst)), msg_type)
        {
            println!(
                "{}",
                serde_json::to_string(&reply_message).expect("json serialize error")
            );
        }
    }
}
