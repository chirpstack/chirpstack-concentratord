use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use gpio_cdev::errors::Error;
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use log::info;

lazy_static! {
    static ref RESET: Mutex<Option<LineHandle>> = Mutex::new(None);
    static ref POWER_EN: Mutex<Option<LineHandle>> = Mutex::new(None);
}

pub fn setup_pins(reset: u32, power_en: Option<u32>) -> Result<(), Error> {
    let mut chip = Chip::new("/dev/gpiochip0")?;

    info!("Configuring reset pin, pin: {}", reset);
    let line = chip.get_line(reset)?;
    let mut reset = RESET.lock().unwrap();
    *reset = Some(line.request(LineRequestFlags::OUTPUT, 0, "reset")?);

    if power_en.is_some() {
        let power_en = power_en.unwrap();
        info!("Configuring power enable pin, pin: {}", power_en);
        let line = chip.get_line(power_en)?;
        let mut power_en = POWER_EN.lock().unwrap();
        *power_en = Some(line.request(LineRequestFlags::OUTPUT, 0, "power_en")?);
    }

    Ok(())
}

pub fn reset() -> Result<(), Error> {
    let reset = RESET.lock().unwrap();
    if reset.is_some() {
        let reset = reset.as_ref().unwrap();

        info!("Triggering concentrator reset");

        reset.set_value(1)?;
        sleep(Duration::from_millis(100));
        reset.set_value(0)?;
        sleep(Duration::from_millis(100));
    }

    let power_en = POWER_EN.lock().unwrap();
    if power_en.is_some() {
        let power_en = power_en.as_ref().unwrap();

        info!("Enabling concentrator power");

        power_en.set_value(1)?;
        sleep(Duration::from_millis(100));
    }

    Ok(())
}
