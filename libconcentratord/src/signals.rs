use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone)]
pub enum Signal {
    Stop,
    Configuration(chirpstack_api::gw::GatewayConfiguration),
}

pub struct SignalPool {
    senders: Vec<Sender<Signal>>,
}

impl SignalPool {
    pub fn new() -> Self {
        SignalPool { senders: vec![] }
    }

    pub fn new_receiver(&mut self) -> Receiver<Signal> {
        let (sender, receiver) = channel();
        self.senders.push(sender);
        return receiver;
    }

    pub fn send_signal(&self, signal: Signal) {
        for s in self.senders.iter() {
            s.send(signal.clone()).unwrap();
        }
    }
}
