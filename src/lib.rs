use std::fmt::Debug;
use std::io::BufRead;

use anyhow::Context;
use node::Node;
use protocol::{Body, InitPayload, Message};
use serde::de::DeserializeOwned;

pub mod node;
pub mod protocol;

pub fn main_loop<N, P>() -> anyhow::Result<()>
where
    N: Node<P>,
    P: DeserializeOwned + Debug,
{
    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut stdout = std::io::stdout().lock();

    let init_msg = serde_json::from_str::<Message<InitPayload>>(
        &stdin
            .next()
            .expect("no init message received")
            .context("failed to read init message from stdin")?,
    )
    .context("init message cloud not be deserialized")?;

    let InitPayload::Init(init) = init_msg.body.payload else {
        panic!("first message should be init payload");
    };

    let reply = Message {
        src: init_msg.dst,
        dst: init_msg.src,
        body: Body {
            id: Some(0),
            in_reply_to: init_msg.body.id,
            payload: InitPayload::InitOk,
        },
    };

    reply.send(&mut stdout)?;

    let mut node = N::from_init(init);

    for line in stdin {
        let line = line.context("input from STDIN cannot be read")?;

        let input = serde_json::from_str::<Message<P>>(&line)
            .context(format!("input from STDIN cannot be deserialized: {}", line))?;

        node.step(input, &mut stdout).context("Node step failed")?;
    }

    Ok(())
}
