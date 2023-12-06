use std::io::StdoutLock;
use std::sync::mpsc::Sender;

use rustrom::main_loop;
use rustrom::node::{Event, Node};
use rustrom::protocol::Init;
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
    fn from_init(init: Init, _inject: Sender<Event<Payload>>) -> Self {
        Self {
            id: init.node_id,
            msg_id: 0,
        }
    }

    fn step(&mut self, input: Event<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let Event::Message(input) = input else {
            panic!("got injected event when there's no event injection");
        };

        let payload = match input.body.payload {
            Payload::Generate => Payload::GenerateOk {
                id: format!("{}-{}", self.id, self.msg_id),
            },
            Payload::GenerateOk { .. } => unreachable!(),
        };

        let reply = input.into_reply_with_payload(Some(&mut self.msg_id), payload);

        reply.send(output)
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<UniqeIdNode, _, _>()
}
