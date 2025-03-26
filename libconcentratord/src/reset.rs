use std::process::Command;
use std::sync::{LazyLock, Mutex};
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use log::info;

type ResetCommand = (String, Vec<String>);

static SX130X_RESET: LazyLock<Mutex<Option<gpiocdev::Request>>> =
    LazyLock::new(|| Mutex::new(None));
static SX1302_POWER_EN: LazyLock<Mutex<Option<gpiocdev::Request>>> =
    LazyLock::new(|| Mutex::new(None));
static SX1261_RESET: LazyLock<Mutex<Option<gpiocdev::Request>>> =
    LazyLock::new(|| Mutex::new(None));
static AD5338R_RESET: LazyLock<Mutex<Option<gpiocdev::Request>>> =
    LazyLock::new(|| Mutex::new(None));
static RESET_COMMANDS: LazyLock<Mutex<Option<Vec<ResetCommand>>>> =
    LazyLock::new(|| Mutex::new(None));

#[derive(Default)]
pub struct Configuration {
    pub sx130x_reset: Option<(String, u32)>,
    pub sx1302_power_en: Option<(String, u32)>,
    pub sx1261_reset: Option<(String, u32)>,
    pub ad5338r_reset: Option<(String, u32)>,
    pub reset_commands: Option<Vec<(String, Vec<String>)>>,
}

pub fn setup_pins(config: Configuration) -> Result<()> {
    if let Some(sx130x_reset) = config.sx130x_reset {
        info!(
            "Configuring reset pin, dev: {}, pin: {}",
            sx130x_reset.0, sx130x_reset.1
        );

        let req = gpiocdev::Request::builder()
            .on_chip(sx130x_reset.0)
            .with_line(sx130x_reset.1)
            .as_output(gpiocdev::line::Value::Inactive)
            .request()?;

        let mut sx130x_reset = SX130X_RESET.lock().unwrap();
        *sx130x_reset = Some(req);
    }

    if let Some(sx1302_power_en) = config.sx1302_power_en {
        info!(
            "Configuring sx1302 power enable pin, dev: {}, pin: {}",
            sx1302_power_en.0, sx1302_power_en.1
        );

        let req = gpiocdev::Request::builder()
            .on_chip(sx1302_power_en.0)
            .with_line(sx1302_power_en.1)
            .as_output(gpiocdev::line::Value::Inactive)
            .request()?;

        let mut sx1302_power_en = SX1302_POWER_EN.lock().unwrap();
        *sx1302_power_en = Some(req);
    }

    if let Some(sx1261_reset) = config.sx1261_reset {
        info!(
            "Configuring sx1261 reset pin, dev: {}, pin: {}",
            sx1261_reset.0, sx1261_reset.1
        );

        let req = gpiocdev::Request::builder()
            .on_chip(sx1261_reset.0)
            .with_line(sx1261_reset.1)
            .as_output(gpiocdev::line::Value::Inactive)
            .request()?;

        let mut sx1261_reset = SX1261_RESET.lock().unwrap();
        *sx1261_reset = Some(req);
    }

    if let Some(ad5338r_reset) = config.ad5338r_reset {
        info!(
            "Configuring ad5338r reset pin, dev: {}, pin: {}",
            ad5338r_reset.0, ad5338r_reset.1
        );

        let req = gpiocdev::Request::builder()
            .on_chip(ad5338r_reset.0)
            .with_line(ad5338r_reset.1)
            .as_output(gpiocdev::line::Value::Inactive)
            .request()?;

        let mut ad5338r_reset = AD5338R_RESET.lock().unwrap();
        *ad5338r_reset = Some(req);
    }

    if let Some(reset_commands) = config.reset_commands {
        info!("Configuring raw reset commands");

        let mut reset_commands_m = RESET_COMMANDS.lock().unwrap();
        *reset_commands_m = Some(reset_commands);
    }

    Ok(())
}

pub fn reset() -> Result<()> {
    let sx1302_power_en = SX1302_POWER_EN.lock().unwrap();
    if sx1302_power_en.is_some() {
        let sx1302_power_en = sx1302_power_en.as_ref().unwrap();

        info!("Enabling concentrator power");

        sx1302_power_en.set_lone_value(gpiocdev::line::Value::Active)?;
        sleep(Duration::from_millis(100));
    }

    let sx130x = SX130X_RESET.lock().unwrap();
    if sx130x.is_some() {
        let sx130x = sx130x.as_ref().unwrap();

        info!("Triggering sx130x reset");

        sx130x.set_lone_value(gpiocdev::line::Value::Active)?;
        sleep(Duration::from_millis(100));
        sx130x.set_lone_value(gpiocdev::line::Value::Inactive)?;
        sleep(Duration::from_millis(100));
    }

    let sx1261_reset = SX1261_RESET.lock().unwrap();
    if sx1261_reset.is_some() {
        let sx1261_reset = sx1261_reset.as_ref().unwrap();

        info!("Triggering sx1261 reset");

        sx1261_reset.set_lone_value(gpiocdev::line::Value::Inactive)?;
        sleep(Duration::from_millis(100));
        sx1261_reset.set_lone_value(gpiocdev::line::Value::Active)?;
        sleep(Duration::from_millis(100));
    }

    let ad5338r_reset = AD5338R_RESET.lock().unwrap();
    if ad5338r_reset.is_some() {
        let ad5338r_reset = ad5338r_reset.as_ref().unwrap();

        info!("Triggering AD5338R reset");
        ad5338r_reset.set_lone_value(gpiocdev::line::Value::Inactive)?;
        sleep(Duration::from_millis(100));
        ad5338r_reset.set_lone_value(gpiocdev::line::Value::Active)?;
        sleep(Duration::from_millis(100));
    }

    let reset_commands = RESET_COMMANDS.lock().unwrap();
    if reset_commands.is_some() {
        let reset_commands = reset_commands.as_ref().unwrap();

        for (cmd, args) in reset_commands {
            info!(
                "Executing reset command, command: {}, args: {:?}",
                cmd, args
            );

            Command::new(cmd).args(args).output()?;
            sleep(Duration::from_millis(100));
        }
    }

    Ok(())
}
