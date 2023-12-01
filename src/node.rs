use crate::protocol::{Init, Message};
use std::io::StdoutLock;

pub trait Node<Payload> {
    fn from_init(init: Init) -> Self
    where
        Self: Sized;
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
}
