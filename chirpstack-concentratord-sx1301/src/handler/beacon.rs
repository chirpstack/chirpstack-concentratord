use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use libconcentratord::jitqueue;
use libconcentratord::signals::Signal;
use libloragw_sx1301::hal;
use uuid::Uuid;

use super::super::{config, wrapper};
use super::{gps, timersync};

const PERIOD: u64 = 128;
const MARGIN: Duration = Duration::from_secs(5);

pub fn beacon_loop(
    conf: &config::Beacon,
    queue: Arc<Mutex<jitqueue::Queue<wrapper::TxPacket>>>,
    stop_receive: Receiver<Signal>,
) {
    debug!("Starting beacon loop");

    loop {
        // Instead of a MARGIN sleep, we receive from the stop channel with a
        // timeout of MARGIN seconds.
        match stop_receive.recv_timeout(MARGIN) {
            Ok(v) => {
                debug!("Received stop signal, signal: {}", v);
                break;
            }
            _ => {}
        };

        let gps_epoch = match gps::get_gps_epoch() {
            Ok(v) => v,
            Err(err) => {
                debug!("Get GPS epoch error, error: {}", err);
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        let next_beacon_time =
            Duration::from_secs(gps_epoch.as_secs() - (gps_epoch.as_secs() % PERIOD) + PERIOD);
        let sleep_time = match next_beacon_time.checked_sub(gps_epoch + MARGIN) {
            Some(v) => v,
            None => continue,
        };

        // Instead of a sleep_time sleep, we receive from the stop channel with a
        // timeout of sleep_time.
        match stop_receive.recv_timeout(sleep_time) {
            Ok(v) => {
                debug!("Received stop signal, signal: {}", v);
                break;
            }
            _ => {}
        };

        match send_beacon(conf, next_beacon_time, &queue) {
            Ok(_) => info!(
                "Beacon enqueued, beacon_time_gps_epoch: {:?}",
                next_beacon_time
            ),
            Err(err) => warn!("Enqueue beacon failed, error: {}", err),
        }
    }

    debug!("Beacon loop ended");
}

fn send_beacon(
    conf: &config::Beacon,
    beacon_time: Duration,
    queue: &Arc<Mutex<jitqueue::Queue<wrapper::TxPacket>>>,
) -> Result<(), String> {
    let mut beacon_pl = get_beacon(conf.compulsory_rfu_size, beacon_time);
    let data_size = beacon_pl.len();

    let mut data: [u8; 256] = [0; 256];
    beacon_pl.resize(data.len(), 0);
    data.copy_from_slice(&beacon_pl);

    let tx_freq = conf.frequencies
        [((beacon_time.as_secs() % (1 << 32)) % conf.frequencies.len() as u64) as usize];
    let tx_packet = hal::TxPacket {
        freq_hz: tx_freq,
        tx_mode: hal::TxMode::OnGPS,
        count_us: match gps::epoch2cnt(&beacon_time) {
            Ok(v) => v,
            Err(err) => return Err(err),
        },
        rf_chain: 0,
        rf_power: conf.tx_power as i8,
        modulation: hal::Modulation::LoRa,
        bandwidth: conf.bandwidth,
        datarate: match conf.spreading_factor {
            7 => hal::DataRate::SF7,
            8 => hal::DataRate::SF8,
            9 => hal::DataRate::SF9,
            10 => hal::DataRate::SF10,
            11 => hal::DataRate::SF11,
            12 => hal::DataRate::SF12,
            _ => return Err("invalid spreading-factor configured".to_string()),
        },
        coderate: hal::CodeRate::LoRa4_5,
        invert_pol: false,
        f_dev: 0,
        preamble: 10,
        no_crc: true,
        no_header: true,
        size: data_size as u16,
        payload: data,
    };
    let tx_packet = wrapper::TxPacket::new(Uuid::new_v4(), tx_packet);

    match queue
        .lock()
        .unwrap()
        .enqueue(timersync::get_concentrator_count(), tx_packet)
    {
        Ok(_) => Ok(()),
        Err(status) => Err(format!("{:?}", status)),
    }
}

fn get_beacon(rfu_size: usize, beacon_time: Duration) -> Vec<u8> {
    // [N: RFU | 4: TIME | 2: CRC]
    let mut b: Vec<u8> = vec![0; rfu_size + 6];
    let beacon_time = beacon_time.as_secs();

    let time_bytes = ((beacon_time % (1 << 32)) as u32).to_le_bytes();
    b[rfu_size..4 + rfu_size].copy_from_slice(&time_bytes);

    let poly: u16 = 0x1021;
    let mut x: u16 = 0;

    for i in 0..b.len() - 2 {
        x ^= (b[i] as u16) << 8;
        for _j in 0..8 {
            if x & 0x8000 != 0 {
                x = (x << 1) ^ poly;
            } else {
                x = x << 1;
            }
        }
    }

    let crc_bytes = x.to_le_bytes();
    b[rfu_size + 4..rfu_size + 6].copy_from_slice(&crc_bytes);

    return b;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_beacon() {
        let beacon_time = Duration::from_secs(0xcc020000);
        let beacon = get_beacon(2, beacon_time);

        assert_eq!(vec![0x00, 0x00, 0x00, 0x00, 0x02, 0xcc, 0xa2, 0x7e], beacon);
    }
}
