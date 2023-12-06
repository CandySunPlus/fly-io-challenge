use std::io::StdoutLock;
use std::sync::mpsc::Sender;

use rustrom::main_loop;
use rustrom::node::{Event, Node};
use rustrom::protocol::Init;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

#[allow(dead_code)]
#[derive(Debug)]
struct EchoNode {
    id: String,
    msg_id: u32,
}

impl Node<Payload> for EchoNode {
    fn from_init(init: Init, _inject: Sender<Event<Payload>>) -> Self {
        EchoNode {
            id: init.node_id,
            msg_id: 0,
        }
    }

    fn step(&mut self, input: Event<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let Event::Message(input) = input else {
            panic!("got injected event when there's no event injection");
        };

        let payload = match input.body.payload {
            Payload::Echo { ref echo } => Payload::EchoOk { echo: echo.clone() },
            Payload::EchoOk { .. } => unreachable!(),
        };
        let reply = input.into_reply_with_payload(Some(&mut self.msg_id), payload);
        reply.send(output)
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<EchoNode, _, _>()
}
