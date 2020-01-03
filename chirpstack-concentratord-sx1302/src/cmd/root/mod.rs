use std::sync::{Arc, Mutex};
use std::thread;

use libconcentratord::{commands, events, jitqueue};

use super::super::{concentrator, config, handler, wrapper};

pub fn run(config: &config::Configuration) -> Result<(), String> {
    info!(
        "Starting Concentratord SX1302 (version: {}, docs: {})",
        config::VERSION,
        "https://www.chirpstack.io/concentratord/"
    );

    // setup concentrator
    concentrator::board_setconf(&config)?;
    concentrator::timestamp_setconf(&config)?;
    concentrator::txgain_setconf(&config)?;
    concentrator::rxrf_setconf(&config)?;
    concentrator::rxif_setconf(&config)?;
    concentrator::start()?;

    // get concentrator eui
    let gateway_id = concentrator::get_eui().unwrap();

    info!(
        "Gateway ID retrieved, gateway_id: {:x?}",
        hex::encode(gateway_id)
    );

    // setup jit queue
    let queue: jitqueue::Queue<wrapper::TxPacket> = jitqueue::Queue::new(32);
    let queue = Arc::new(Mutex::new(queue));

    // setup zeromq
    events::bind_socket(&config.concentratord.api.event_bind).expect("bind event socket error");
    let rep_sock = commands::get_socket(&config.concentratord.api.command_bind)
        .expect("bind command socket error");

    // uplink thread
    let up_thread = thread::spawn({
        let gateway_id = gateway_id.clone();

        move || {
            handler::uplink::handle_loop(&gateway_id);
        }
    });

    // jit thread
    let jit_thread = thread::spawn({
        let queue = Arc::clone(&queue);
        let antenna_gain = config.gateway.antenna_gain;

        move || {
            handler::jit::jit_loop(queue, antenna_gain);
        }
    });

    // command thread
    let command_thread = thread::spawn({
        let vendor_config = config.gateway.model_config.clone();
        let gateway_id = gateway_id.clone();

        move || {
            handler::command::handle_loop(&vendor_config, &gateway_id, queue, rep_sock);
        }
    });

    // gps thread
    let gps_thread = thread::spawn({
        let gps_tty_path = config.gateway.gps_tty_path.clone();

        move || {
            handler::gps::gps_loop(&gps_tty_path);
        }
    });

    // gps validate thread
    let gps_validate_thread = thread::spawn({
        let gps_tty_path = config.gateway.gps_tty_path.clone();

        move || {
            handler::gps::gps_validate_loop(&gps_tty_path);
        }
    });

    // stats thread
    let stats_thread = thread::spawn({
        let gateway_id = gateway_id.clone();
        let stats_interval = config.concentratord.stats_interval;

        move || {
            handler::stats::stats_loop(&gateway_id, &stats_interval);
        }
    });

    up_thread.join().unwrap();
    jit_thread.join().unwrap();
    command_thread.join().unwrap();
    gps_thread.join().unwrap();
    gps_validate_thread.join().unwrap();
    stats_thread.join().unwrap();

    return Ok(());
}
