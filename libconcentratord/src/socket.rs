use std::sync::{LazyLock, Mutex};

pub static ZMQ_CONTEXT: LazyLock<Mutex<zmq::Context>> =
    LazyLock::new(|| Mutex::new(zmq::Context::new()));
