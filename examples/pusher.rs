#[macro_use] extern crate log;
extern crate env_logger;
extern crate scaproust;

use std::io;

use scaproust::{Session, SocketType, Socket};

fn handle_comand(cmd: &str, socket: &mut Socket) {
	println!("User command: {:?}", cmd);
    match socket.send(vec!(66, 67, 68, 69)) {
        Ok(_) => info!("message sent !"),
        Err(e) => error!("message NOT sent: {} !", e)
    }
}

fn main() {

    env_logger::init().unwrap();
    info!("Logging initialized.");

    let session = Session::new().unwrap();
    let mut socket = session.create_socket(SocketType::Push).unwrap();

    assert!(socket.connect("tcp://127.0.0.1:5454").is_ok());
    assert!(socket.connect("tcp://127.0.0.1:5455").is_ok());
    assert!(socket.connect("tcp://some random crap").is_err());

    let mut input = String::new();
    loop {
		match io::stdin().read_line(&mut input) {
		    Ok(0) => return,
		    Ok(_) => handle_comand(&input ,&mut socket),
		    Err(error) => println!("error: {}", error),
		};
		input.clear();
    }
}
