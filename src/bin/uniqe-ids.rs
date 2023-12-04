use std::io::StdoutLock;

use rustrom::main_loop;
use rustrom::node::Node;
use rustrom::protocol::{Init, Message};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Payload {
    Generate,
    GenerateOk { id: String },
}

#[allow(dead_code)]
struct UniqeIdNode {
    id: String,
    msg_id: u32,
}

impl Node<Payload> for UniqeIdNode {
    fn from_init(init: Init) -> Self {
        Self {
            id: init.node_id,
            msg_id: 0,
        }
    }

    fn step(&mut self, input: Message<Payload>, mut output: &mut StdoutLock) -> anyhow::Result<()> {
        let payload = match input.body.payload {
            Payload::Generate => Payload::GenerateOk {
                id: format!("{}-{}", self.id, self.msg_id),
            },
            Payload::GenerateOk { .. } => unreachable!(),
        };

        let reply = input.into_reply_with_payload(Some(&mut self.msg_id), payload);

        reply.send(&mut output)
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<UniqeIdNode, _>()
}
