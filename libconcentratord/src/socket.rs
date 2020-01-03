use std::sync::Mutex;

lazy_static! {
    pub static ref ZMQ_CONTEXT: Mutex<zmq::Context> = Mutex::new(zmq::Context::new());
}
