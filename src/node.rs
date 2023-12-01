use crate::protocol::{Init, Message};
use std::io::StdoutLock;

pub trait Node<Payload> {
    fn from_init(init: Init) -> Self;
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
}
