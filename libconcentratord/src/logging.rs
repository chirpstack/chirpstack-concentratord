use std::process;

use syslog::{BasicLogger, Facility, Formatter3164};

pub fn setup(name: &str, level: log::Level, syslog: bool) -> Result<(), String> {
    if syslog {
        let formatter = Formatter3164 {
            facility: Facility::LOG_USER,
            hostname: None,
            process: name.to_string(),
            pid: process::id() as i32,
        };
        let logger = match syslog::unix(formatter) {
            Ok(v) => v,
            Err(err) => return Err(format!("create syslog logger error: {}", err).to_string()),
        };

        log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
            .map(|()| log::set_max_level(level.to_level_filter()))
            .unwrap();
    } else {
        simple_logger::init_with_level(level).unwrap();
    }

    return Ok(());
}
