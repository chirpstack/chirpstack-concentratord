use std::sync::Mutex;

lazy_static! {
    pub static ref CONCENTATOR: Mutex<()> = Mutex::new(());
}
