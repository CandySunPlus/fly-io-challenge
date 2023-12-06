use std::io::StdoutLock;
use std::sync::mpsc::Sender;

use crate::protocol::{Init, Message};

pub enum Event<Payload, InjectedPayload = ()> {
    Message(Message<Payload>),
    Injected(InjectedPayload),
    EOF,
}

pub trait Node<Payload, InjectedPayload = ()> {
    fn from_init(init: Init, inject: Sender<Event<Payload, InjectedPayload>>) -> Self
    where
        Self: Sized;
    fn step(
        &mut self,
        input: Event<Payload, InjectedPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()>;
}
