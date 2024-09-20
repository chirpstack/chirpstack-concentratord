use super::super::super::super::config;
use super::super::Configuration;

pub fn new(conf: &config::Configuration) -> Configuration {
    Configuration {
        tty_path: conf.gateway.get_com_dev_path("/dev/ttyACM0"),
        min_max_tx_freq: (2400000000, 2483500000),
        // pin configuration taken from:
        // https://github.com/Lora-net/gateway_2g4_hal/blob/master/tools/rpi_configure_gpio.sh
        reset_pin: conf.gateway.get_mcu_reset_pin("/dev/gpiochip0", 32),
        boot0_pin: conf.gateway.get_mcu_boot_pin("/dev/gpiochip0", 18),
    }
}
