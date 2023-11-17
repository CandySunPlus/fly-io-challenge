use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
}

impl MessageType {
    pub fn reply_type(&self) -> MessageType {
        match self {
            MessageType::Echo { echo } => MessageType::EchoOk {
                echo: echo.to_owned(),
            },
            MessageType::Init {
                node_id: _,
                node_ids: _,
            } => MessageType::InitOk {},
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageBody {
    #[serde(flatten)]
    r#type: MessageType,
    msg_id: Option<u32>,
    in_reply_to: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    src: String,
    dest: String,
    body: MessageBody,
}

impl Message {
    pub fn r#type(&self) -> &MessageType {
        &self.body.r#type
    }

    pub fn id(&self) -> Option<u32> {
        self.body.msg_id
    }

    pub fn reply(&self) -> Message {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: MessageBody {
                r#type: self.body.r#type.reply_type(),
                msg_id: self.body.msg_id,
                in_reply_to: self.body.msg_id,
            },
        }
    }
}
