use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use gpio_cdev::errors::Error;
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use log::info;

lazy_static! {
    static ref RESET: Mutex<Option<LineHandle>> = Mutex::new(None);
}

pub fn setup_pins(reset: u32) -> Result<(), Error> {
    info!("Configuring reset pin, pin: {}", reset);
    let mut chip = Chip::new("/dev/gpiochip0")?;
    let line = chip.get_line(reset)?;

    let mut reset = RESET.lock().unwrap();
    *reset = Some(line.request(LineRequestFlags::OUTPUT, 0, "reset")?);
    Ok(())
}

pub fn reset() -> Result<(), Error> {
    let reset = RESET.lock().unwrap();
    if reset.is_none() {
        return Ok(());
    }
    let reset = reset.as_ref().unwrap();

    info!("Triggering concentrator reset");

    reset.set_value(1)?;
    sleep(Duration::from_millis(100));
    reset.set_value(0)?;
    sleep(Duration::from_millis(100));
    Ok(())
}
