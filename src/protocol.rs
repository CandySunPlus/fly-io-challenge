use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageType {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {},
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Generate {},
    GenerateOk {
        id: String,
    },
    Broadcast {
        message: u32,
    },
    BroadcastOk {},
    Read {},
    ReadOk {
        messages: Vec<u32>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk {},
    Error {
        code: u32,
        text: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageBody {
    #[serde(flatten)]
    r#type: MessageType,
    msg_id: Option<u32>,
    in_reply_to: Option<u32>,
}

impl MessageBody {
    pub fn new(r#type: MessageType, msg_id: Option<u32>, in_reply_to: Option<u32>) -> Self {
        Self {
            r#type,
            msg_id,
            in_reply_to,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    src: String,
    dest: String,
    body: MessageBody,
}

impl Message {
    pub fn new(src: &str, dest: &str, body: MessageBody) -> Self {
        Self {
            src: src.to_owned(),
            dest: dest.to_owned(),
            body,
        }
    }

    pub fn r#type(&self) -> &MessageType {
        &self.body.r#type
    }

    pub fn id(&self) -> Option<u32> {
        self.body.msg_id
    }

    pub fn src(&self) -> &str {
        &self.src
    }

    pub fn dest(&self) -> &str {
        &self.dest
    }

    pub fn create_reply_msg(&self, msg_id: Option<u32>, msg_type: MessageType) -> Option<Message> {
        if self.id().is_none() {
            None
        } else {
            Some(Message {
                src: self.dest.clone(),
                dest: self.src.clone(),
                body: MessageBody {
                    r#type: msg_type,
                    msg_id,
                    in_reply_to: self.id(),
                },
            })
        }
    }
}
