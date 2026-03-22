use std::io::{BufRead, BufReader};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use anyhow::Result;

use libconcentratord::{gnss, gpsd, signals::Signal};
use libloragw_sx1301::gps;

use crate::handler::timersync;

pub fn gps_loop(gps_device: gnss::Device, stop_receive: Receiver<Signal>) -> Result<()> {
    debug!("Starting GPS loop");

    let mut gps_reader: Box<dyn BufRead> = match gps_device {
        gnss::Device::TtyPath(tty_path) => {
            info!("Enabling GPS device, tty_path: {}", tty_path);
            let gps_file = gps::enable(&tty_path, gps::GPSFamily::UBX7, 0)
                .expect("could not open gps tty path for gps sync");
            Box::new(BufReader::new(gps_file)) as Box<dyn BufRead>
        }
        gnss::Device::Gpsd(gpsd_host) => {
            info!("Starting gpsd reader, server: localhost:2947");
            Box::new(gpsd::get_reader(&gpsd_host).expect("could not open gpsd reader"))
                as Box<dyn BufRead>
        }
        gnss::Device::None => {
            warn!("No GPS device configured");
            return Ok(());
        }
    };

    loop {
        if let Ok(v) = stop_receive.recv_timeout(Duration::from_millis(0)) {
            debug!("Received stop signal, signal: {}", v);
            return Ok(());
        }

        if let Ok(v) = gnss::read(&mut gps_reader)
            && let Some(v) = v
        {
            gnss::sync(&v, timersync::get_concentrator_count())?;
        }
    }
}
