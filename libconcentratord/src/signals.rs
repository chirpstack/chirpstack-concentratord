use std::fmt;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Clone)]
pub enum Signal {
    Stop,
    Configuration(chirpstack_api::gw::GatewayConfiguration),
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Signal::Stop => write!(f, "Stop"),
            Signal::Configuration(_) => write!(f, "Configuration"),
        }
    }
}

#[derive(Default)]
pub struct SignalPool {
    senders: Vec<Sender<Signal>>,
}

impl SignalPool {
    pub fn new_receiver(&mut self) -> Receiver<Signal> {
        let (sender, receiver) = channel();
        self.senders.push(sender);
        receiver
    }

    pub fn send_signal(&self, signal: Signal) {
        for s in self.senders.iter() {
            s.send(signal.clone()).unwrap();
        }
    }
}
