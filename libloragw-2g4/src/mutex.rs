use std::sync::{LazyLock, Mutex};

pub static CONCENTATOR: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
