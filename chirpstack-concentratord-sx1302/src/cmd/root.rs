use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::Result;
use libconcentratord::signals;
use libconcentratord::signals::Signal;
use libconcentratord::{commands, events, jitqueue, reset};
use libloragw_sx1302::hal;

use crate::config::vendor::ComType;
use crate::{concentrator, config, handler, wrapper};

pub fn run(
    config: &config::Configuration,
    stop_send: Sender<Signal>,
    stop_receive: Rc<Receiver<Signal>>,
) -> Result<Signal> {
    info!(
        "Starting Concentratord SX1302 (version: {}, docs: {})",
        config::VERSION,
        "https://www.chirpstack.io/docs/chirpstack-concentratord/"
    );

    // reset concentrator
    reset::reset().expect("concentrator reset failed");

    // setup concentrator
    concentrator::set_i2c_device_path(config)?;
    concentrator::set_i2c_temp_sensor_addr(config)?;
    concentrator::board_setconf(config)?;
    concentrator::timestamp_setconf(config)?;
    concentrator::txgain_setconf(config)?;
    concentrator::rxrf_setconf(config)?;
    concentrator::rxif_setconf(config)?;
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
    let queue: jitqueue::Queue<wrapper::TxPacket> =
        jitqueue::Queue::new(32, config.get_duty_cycle_tracker());
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

            debug!("Uplink loop ended");
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
        let vendor_config = config.gateway.model_config.clone();
        let queue = Arc::clone(&queue);
        let stop_receive = signal_pool.new_receiver();
        let stop_send = stop_send.clone();
        let stop_send_err = stop_send.clone();

        move || {
            if let Err(e) = handler::command::handle_loop(
                &vendor_config,
                &gateway_id,
                queue,
                rep_sock,
                stop_receive,
                stop_send,
            ) {
                error!("Command handle loop error: {}", e);
                stop_send_err.send(Signal::Stop).unwrap();
            }

            debug!("Command handle loop ended");
        }
    }));

    // stats thread
    threads.push(thread::spawn({
        let stats_interval = config.concentratord.stats_interval;
        let queue = Arc::clone(&queue);

        // In case of USB, there is no I2C configuration.
        let get_temperature = config.gateway.model_config.com_type == ComType::Usb
            || (config.gateway.model_config.com_type == ComType::Spi
                && config.gateway.model_config.i2c_temp_sensor_addr.is_some());

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
                get_temperature,
                &stats_interval,
                stop_receive,
                metadata,
                queue,
            ) {
                error!("Stats loop error: {}", e);
                stop_send.send(Signal::Stop).unwrap();
            }

            debug!("Stats loop ended");
        }
    }));

    if config.gateway.model_config.gps != config::vendor::Gps::None {
        // gps thread
        threads.push(thread::spawn({
            let gps = config.gateway.model_config.gps.clone();
            let stop_receive = signal_pool.new_receiver();
            let stop_send = stop_send.clone();

            move || {
                if let Err(e) = handler::gps::gps_loop(gps, stop_receive) {
                    error!("GPS loop error: {}", e);
                    stop_send.send(Signal::Stop).unwrap();
                }

                debug!("GPS loop ended")
            }
        }));

        // gps validate thread
        threads.push(thread::spawn({
            let stop_receive = signal_pool.new_receiver();

            move || {
                handler::gps::gps_validate_loop(stop_receive);
            }
        }));

        // beacon thread
        if !config.gateway.beacon.frequencies.is_empty() {
            threads.push(thread::spawn({
                let beacon_config = config.gateway.beacon.clone();
                let queue = Arc::clone(&queue);
                let stop_receive = signal_pool.new_receiver();
                let stop_send = stop_send.clone();

                move || {
                    if let Err(e) =
                        handler::beacon::beacon_loop(&beacon_config, queue, stop_receive)
                    {
                        error!("Beacon loop error: {}", e);
                        stop_send.send(Signal::Stop).unwrap();
                    }

                    debug!("Beacon loop ended");
                }
            }));
        }
    }

    let stop_signal = stop_receive.recv().unwrap();
    signal_pool.send_signal(stop_signal.clone());

    for t in threads {
        t.join().unwrap();
    }

    concentrator::stop()?;

    Ok(stop_signal)
}
