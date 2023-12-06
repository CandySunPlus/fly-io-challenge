use std::fmt::Debug;
use std::io::BufRead;
use std::sync::mpsc;
use std::thread;

use anyhow::Context;
use node::{Event, Node};
use protocol::{Body, InitPayload, Message};
use serde::de::DeserializeOwned;

pub mod node;
pub mod protocol;

pub fn main_loop<N, P, IP>() -> anyhow::Result<()>
where
    N: Node<P, IP>,
    P: DeserializeOwned + Debug + Send + 'static,
    IP: Send + 'static,
{
    let (tx, rx) = mpsc::channel();

    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut stdout = std::io::stdout().lock();

    let init_line = stdin
        .next()
        .expect("no init message received")
        .context("failed to read init message from stdin")?;

    let init_msg = serde_json::from_str::<Message<InitPayload>>(&init_line)
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

    let mut node = N::from_init(init, tx.clone());

    drop(stdin);

    let jh = thread::spawn(move || {
        let stdin = std::io::stdin().lock();
        for line in stdin.lines() {
            let line = line.context("input from STDIN cannot be read")?;
            let input = serde_json::from_str::<Message<P>>(&line)
                .context(format!("input from STDIN cannot be deserialized: {}", line))?;
            if tx.send(Event::Message(input)).is_err() {
                return Ok::<_, anyhow::Error>(());
            }
        }
        let _ = tx.send(Event::EOF);
        Ok(())
    });

    for input in rx {
        node.step(input, &mut stdout).context("Node step failed")?;
    }

    jh.join()
        .expect("stdin thread panicked")
        .context("stdin thread err")?;

    Ok(())
}
