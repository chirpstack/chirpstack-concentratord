use libconcentratord::commands::get_client;

pub fn get_gateway_id(command_url: &str) -> Result<Vec<u8>, String> {
    debug!("Reading gateway id, server: {}", command_url);

    let sock = get_client(command_url).expect("get client error");

    // send 'gateway_id' command with empty payload
    sock.send("gateway_id", zmq::SNDMORE).unwrap();
    sock.send("", 0).unwrap();

    // set poller so that we can timout after 100ms
    let mut items = [sock.as_poll_item(zmq::POLLIN)];
    zmq::poll(&mut items, 100).unwrap();
    if !items[0].is_readable() {
        return Err("could not read gateway_id".to_string());
    }

    // read 'gateway_id' response
    let gateway_id = sock.recv_bytes(0).unwrap();
    return Ok(gateway_id);
}
