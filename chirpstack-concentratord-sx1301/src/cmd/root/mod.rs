use std::sync::{Arc, Mutex};
use std::thread;

use super::super::{concentrator, config, handler, wrapper};

pub fn run(config: config::Configuration) -> Result<(), String> {
    info!(
        "Starting Concentratord SX1301 (version: {}, docs: {})",
        config::VERSION,
        "https://www.chirpstack.io/concentratord/"
    );

    // setup concentrator
    concentrator::board_setconf(&config)?;
    concentrator::txgain_setconf(&config)?;
    concentrator::rxrf_setconf(&config)?;
    concentrator::rxif_setconf(&config)?;
    concentrator::start(&config)?;

    // setup jit queue
    let queue: jitqueue::Queue<wrapper::TxPacket> = jitqueue::Queue::new(32);
    let queue = Arc::new(Mutex::new(queue));

    // setup zeromq
    let zmq_ctx = zmq::Context::new();

    // uplink thread
    let up_handler = thread::spawn({
        let gateway_id = config.gateway.gateway_id_bytes.clone();
        let zmq_pub = zmq_ctx.socket(zmq::PUB).unwrap();
        zmq_pub
            .bind(&config.concentratord.api.event_bind)
            .expect("bind event socket error");

        move || {
            handler::uplink::handle_loop(&gateway_id, zmq_pub);
        }
    });

    // timer sync thread
    let time_sync = thread::spawn({
        move || {
            handler::timersync::timesync_loop();
        }
    });

    // jit thread
    let jit_loop = thread::spawn({
        let queue = Arc::clone(&queue);
        let antenna_gain = config.gateway.antenna_gain;

        move || {
            handler::jit::jit_loop(queue, antenna_gain);
        }
    });

    // gateway command thread
    let command_loop = thread::spawn({
        let vendor_config = config.gateway.model_config.clone();
        let gateway_id = config.gateway.gateway_id_bytes.clone();
        let queue = Arc::clone(&queue);
        let zmq_rep = zmq_ctx.socket(zmq::REP).unwrap();
        zmq_rep
            .bind(&config.concentratord.api.command_bind)
            .expect("bind command socket error");

        move || {
            handler::command::handle_loop(&vendor_config, &gateway_id, zmq_rep, queue);
        }
    });

    // gps thread
    let gps_loop = thread::spawn({
        let gps_tty_path = config.gateway.gps_tty_path.clone();

        move || {
            handler::gps::gps_loop(&gps_tty_path);
        }
    });

    // gps validate thread
    let gps_validate_loop = thread::spawn({
        let gps_tty_path = config.gateway.gps_tty_path.clone();

        move || {
            handler::gps::gps_validate_loop(&gps_tty_path);
        }
    });

    // stats thread
    let stats_loop = thread::spawn({
        let gateway_id = config.gateway.gateway_id_bytes.clone();
        let stats_interval = config.concentratord.stats_interval;
        let zmq_pub = zmq_ctx.socket(zmq::PUB).unwrap();
        zmq_pub
            .bind(&config.concentratord.api.event_bind)
            .expect("bind event socket error");

        move || {
            handler::stats::stats_loop(&gateway_id, &stats_interval, zmq_pub);
        }
    });

    up_handler.join().unwrap();
    time_sync.join().unwrap();
    jit_loop.join().unwrap();
    command_loop.join().unwrap();
    gps_loop.join().unwrap();
    gps_validate_loop.join().unwrap();
    stats_loop.join().unwrap();

    return Ok(());
}
