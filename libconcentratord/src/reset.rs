use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use gpio_cdev::errors::Error;
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use log::info;

lazy_static! {
    static ref SX1302_RESET: Mutex<Option<LineHandle>> = Mutex::new(None);
    static ref SX1302_POWER_EN: Mutex<Option<LineHandle>> = Mutex::new(None);
    static ref SX1261_RESET: Mutex<Option<LineHandle>> = Mutex::new(None);
}

pub fn setup_pins(
    sx1302_reset: (u32, u32),
    sx1302_power_en: Option<(u32, u32)>,
    sx1261_reset: Option<(u32, u32)>,
) -> Result<(), Error> {
    info!(
        "Configuring reset pin, chip: {}, pin: {}",
        sx1302_reset.0, sx1302_reset.1
    );

    let mut chip = Chip::new(format!("/dev/gpiochip{}", sx1302_reset.0))?;
    let line = chip.get_line(sx1302_reset.1)?;
    let mut sx1302_reset = SX1302_RESET.lock().unwrap();
    *sx1302_reset = Some(line.request(LineRequestFlags::OUTPUT, 0, "sx1302_reset")?);

    if sx1302_power_en.is_some() {
        let sx1302_power_en = sx1302_power_en.unwrap();

        info!(
            "Configuring sx1302 power enable pin, chip: {}, pin: {}",
            sx1302_power_en.0, sx1302_power_en.1
        );

        let mut chip = Chip::new(format!("/dev/gpiochip{}", sx1302_power_en.0))?;
        let line = chip.get_line(sx1302_power_en.1)?;
        let mut sx1302_power_en = SX1302_POWER_EN.lock().unwrap();
        *sx1302_power_en = Some(line.request(LineRequestFlags::OUTPUT, 0, "sx1302_power_en")?);
    }

    if sx1261_reset.is_some() {
        let sx1261_reset = sx1261_reset.unwrap();

        info!(
            "Configuring sx1261 reset pin, chip: {}, pin: {}",
            sx1261_reset.0, sx1261_reset.1
        );

        let mut chip = Chip::new(format!("/dev/gpiochip{}", sx1261_reset.0))?;
        let line = chip.get_line(sx1261_reset.1)?;
        let mut sx1261_reset = SX1261_RESET.lock().unwrap();
        *sx1261_reset = Some(line.request(LineRequestFlags::OUTPUT, 0, "sx1261_reset")?);
    }

    Ok(())
}

pub fn reset() -> Result<(), Error> {
    let sx1302 = SX1302_RESET.lock().unwrap();
    if sx1302.is_some() {
        let sx1302 = sx1302.as_ref().unwrap();

        info!("Triggering sx1302 reset");

        sx1302.set_value(1)?;
        sleep(Duration::from_millis(100));
        sx1302.set_value(0)?;
        sleep(Duration::from_millis(100));
    }

    let sx1302_power_en = SX1302_POWER_EN.lock().unwrap();
    if sx1302_power_en.is_some() {
        let sx1302_power_en = sx1302_power_en.as_ref().unwrap();

        info!("Enabling concentrator power");

        sx1302_power_en.set_value(1)?;
        sleep(Duration::from_millis(100));
    }

    let sx1261_reset = SX1261_RESET.lock().unwrap();
    if sx1261_reset.is_some() {
        let sx1261_reset = sx1261_reset.as_ref().unwrap();

        info!("Triggering sx1261 reset");

        sx1261_reset.set_value(1)?;
        sleep(Duration::from_millis(100));
        sx1261_reset.set_value(0)?;
        sleep(Duration::from_millis(100));
    }

    Ok(())
}
