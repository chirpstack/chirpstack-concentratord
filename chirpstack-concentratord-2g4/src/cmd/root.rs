use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::Result;
use libconcentratord::signals;
use libconcentratord::signals::Signal;
use libconcentratord::{commands, events, jitqueue, reset};
use libloragw_2g4::hal;

use super::super::{concentrator, config, handler, wrapper};

pub fn run(
    config: &config::Configuration,
    stop_send: Sender<Signal>,
    stop_receive: Rc<Receiver<Signal>>,
) -> Result<Signal> {
    info!(
        "Starting Concentratord 2g4 (version: {}, docs: {})",
        config::VERSION,
        "https://www.chirpstack.io/docs/chirpstack-concentratord/"
    );

    // reset concentrator
    reset::reset().expect("concentrator reset failed");

    // setup concentrator
    concentrator::board_setconf(config)?;
    concentrator::rx_setconf(config)?;
    concentrator::tx_setconf(config)?;
    concentrator::start()?;

    // setup static location
    handler::gps::set_static_gps_coords(
        config.gateway.location.latitude,
        config.gateway.location.longitude,
        config.gateway.location.altitude,
    );

    // get concentrator eui
    let gateway_id = concentrator::get_eui().unwrap();

    info!(
        "Gateway ID retrieved, gateway_id: {:x?}",
        hex::encode(gateway_id)
    );

    // setup jit queue
    let queue: jitqueue::Queue<wrapper::TxPacket> = jitqueue::Queue::new(32, None);
    let queue = Arc::new(Mutex::new(queue));

    // setup zeromq
    events::bind_socket(&config.concentratord.api.event_bind).expect("bind event socket error");
    let rep_sock = commands::get_socket(&config.concentratord.api.command_bind)
        .expect("bind command socket error");

    // setup threads
    let mut signal_pool = signals::SignalPool::default();
    let mut threads: Vec<thread::JoinHandle<()>> = vec![];

    // uplink thread
    threads.push(thread::spawn({
        let stop_receive = signal_pool.new_receiver();
        let stop_send = stop_send.clone();
        let disable_crc_filter = config.concentratord.disable_crc_filter;
        let time_fallback = config.gateway.time_fallback_enabled;

        move || {
            if let Err(e) = handler::uplink::handle_loop(
                &gateway_id,
                stop_receive,
                disable_crc_filter,
                time_fallback,
            ) {
                error!("Uplink loop error: {}", e);
                stop_send.send(Signal::Stop).unwrap();
            }

            debug!("Uplink handle loop ended");
        }
    }));

    // jit thread
    threads.push(thread::spawn({
        let queue = Arc::clone(&queue);
        let antenna_gain_dbi = config.gateway.antenna_gain;
        let stop_receive = signal_pool.new_receiver();
        let stop_send = stop_send.clone();

        move || {
            if let Err(e) = handler::jit::jit_loop(queue, antenna_gain_dbi, stop_receive) {
                error!("JIT loop error: {}", e);
                stop_send.send(Signal::Stop).unwrap();
            }

            debug!("JIT loop ended");
        }
    }));

    // command thread
    threads.push(thread::spawn({
        let queue = Arc::clone(&queue);
        let vendor_config = config.gateway.model_config.clone();
        let stop_receive = signal_pool.new_receiver();
        let stop_send = stop_send.clone();
        let stop_send_err = stop_send.clone();
        let lorawan_public = config.gateway.lorawan_public;

        move || {
            if let Err(e) = handler::command::handle_loop(
                lorawan_public,
                &vendor_config,
                &gateway_id,
                queue,
                rep_sock,
                stop_receive,
                stop_send,
            ) {
                error!("Command loop error: {}", e);
                stop_send_err.send(Signal::Stop).unwrap();
            }

            debug!("Command loop ended");
        }
    }));

    // stats thead
    threads.push(thread::spawn({
        let stats_interval = config.concentratord.stats_interval;
        let queue = Arc::clone(&queue);
        let stop_receive = signal_pool.new_receiver();
        let stop_send = stop_send.clone();
        let mut metadata = HashMap::new();
        metadata.insert(
            "config_version".to_string(),
            config.gateway.config_version.clone(),
        );
        metadata.insert(
            "concentratord_version".to_string(),
            config::VERSION.to_string(),
        );
        metadata.insert("model".to_string(), config.gateway.model.clone());
        metadata.insert("hal_version".to_string(), hal::version_info());

        move || {
            if let Err(e) = handler::stats::stats_loop(
                &gateway_id,
                &stats_interval,
                stop_receive,
                &metadata,
                queue,
            ) {
                error!("Stats loop error: {}", e);
                stop_send.send(Signal::Stop).unwrap();
            }

            debug!("Stats loop ended");
        }
    }));

    let stop_signal = stop_receive.recv().unwrap();
    signal_pool.send_signal(stop_signal.clone());

    for t in threads {
        t.join().unwrap();
    }

    concentrator::stop()?;

    Ok(stop_signal)
}
