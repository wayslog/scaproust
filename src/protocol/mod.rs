use std::rc::Rc;
use std::sync::mpsc;
use std::io;

use mio;

use global::SocketType as SocketType;
use event_loop_msg::SocketEvt;
use pipe::Pipe as Pipe;
use EventLoop;
use Message;

pub mod push;
pub mod pull;

pub trait Protocol {
	fn id(&self) -> u16;
	fn peer_id(&self) -> u16;

	fn add_pipe(&mut self, id: usize, pipe: Pipe);
	fn remove_pipe(&mut self, id: usize) -> Option<String>;

	fn ready(&mut self, event_loop: &mut EventLoop, id: usize, events: mio::EventSet) -> io::Result<()>;
	fn send(&mut self, event_loop: &mut EventLoop, msg: Message);
}


pub fn create_protocol(socket_type: SocketType, evt_tx: Rc<mpsc::Sender<SocketEvt>>) -> Box<Protocol> {
	match socket_type {
		SocketType::Push => Box::new(push::Push::new(evt_tx)),
		SocketType::Pull => Box::new(pull::Pull),
		_ => panic!("")
	}
}
