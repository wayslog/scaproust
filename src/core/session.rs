// Copyright 2016 Benoît Labaere (benoit.labaere@gmail.com)
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>
// or the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// This file may not be copied, modified, or distributed except according to those terms.

use std::rc::Rc;
use std::collections::HashMap;

use std::sync::mpsc;

use mio;

use global::*;
use event_loop_msg::*;
use protocol;
use core::socket::Socket;
use core::probe::Probe;

use EventLoop;

pub struct Session {
    event_sender: mpsc::Sender<SessionNotify>,
    sockets: HashMap<SocketId, Socket>,
    socket_ids: HashMap<mio::Token, SocketId>,
    probes: HashMap<ProbeId, Probe>,
    probe_ids: HashMap<SocketId, ProbeId>,
    id_seq: IdSequence
}

impl Session {

    pub fn new(event_tx: mpsc::Sender<SessionNotify>) -> Session {
        Session {
            event_sender: event_tx,
            sockets: HashMap::new(),
            socket_ids: HashMap::new(),
            probes: HashMap::new(),
            probe_ids: HashMap::new(),
            id_seq: IdSequence::new()
        }
    }

    fn handle_cmd(&mut self, event_loop: &mut EventLoop, cmd: CmdSignal) {
        debug!("session handle_cmd {}", cmd.name());
        match cmd {
            CmdSignal::Session(c)    => self.handle_session_cmd(event_loop, c),
            CmdSignal::Socket(id, c) => self.handle_socket_cmd(event_loop, id, c),
            CmdSignal::Probe(id, c) =>  self.handle_probe_cmd(event_loop, id, c)
        }
    }

    fn handle_session_cmd(&mut self, event_loop: &mut EventLoop, cmd: SessionCmdSignal) {
        debug!("session handle_session_cmd {}", cmd.name());
        match cmd {
            SessionCmdSignal::CreateSocket(t)   => self.create_socket(event_loop, t),
            SessionCmdSignal::DestroySocket(id) => self.destroy_socket(event_loop, id),
            SessionCmdSignal::CreateProbe(l,r)  => self.create_probe(event_loop, l, r),
            SessionCmdSignal::DestroyProbe(id)  => self.destroy_probe(event_loop, id),
            SessionCmdSignal::Shutdown          => {
                self.socket_ids.clear();
                self.sockets.clear();
                event_loop.shutdown();
            },
        }
    }

    fn handle_socket_cmd(&mut self, event_loop: &mut EventLoop, id: SocketId, cmd: SocketCmdSignal) {
        debug!("session handle_socket_cmd {}", cmd.name());
        self.on_socket_by_id(&id, |s| s.handle_cmd(event_loop, cmd));
    }

    fn handle_probe_cmd(&mut self, event_loop: &mut EventLoop, id: ProbeId, cmd: ProbeCmdSignal) {
        debug!("session handle_probe_cmd {}", cmd.name());
    }

    fn handle_evt(&mut self, event_loop: &mut EventLoop, evt: EvtSignal) {
        debug!("session handle_evt {}", evt.name());
        match evt {
            EvtSignal::Socket(id, e) => self.handle_socket_evt(event_loop, id, e),
            EvtSignal::Pipe(tok, e)  => self.handle_pipe_evt(event_loop, tok, e)
        }
    }

    fn handle_socket_evt(&mut self, event_loop: &mut EventLoop, id: SocketId, evt: SocketEvtSignal) {
        debug!("session handle_socket_evt {}", evt.name());
        match evt {
            SocketEvtSignal::PipeAdded(tok) => {
                self.socket_ids.insert(tok, id);
                self.on_socket_by_id(&id, |s| s.handle_evt(event_loop, SocketEvtSignal::PipeAdded(tok)));
            },
            SocketEvtSignal::AcceptorAdded(tok) => {
                self.socket_ids.insert(tok, id);
                self.on_socket_by_id(&id, |s| s.handle_evt(event_loop, SocketEvtSignal::AcceptorAdded(tok)));
            },
            SocketEvtSignal::Readable => {
                if let Some(probe_id) = self.probe_ids.get(&id) {
                    if let Some(probe) = self.probes.get_mut(probe_id) {
                        probe.on_socket_readable(id);
                    }
                }
            }
        }
    }

    fn handle_pipe_evt(&mut self, event_loop: &mut EventLoop, tok: mio::Token, evt: PipeEvtSignal) {
        debug!("session handle_pipe_evt {}", evt.name());

        match evt {
            PipeEvtSignal::Closed => {
                self.socket_ids.remove(&tok);
            },
            other => {
                self.on_socket_by_token(&tok, |s| s.on_pipe_evt(event_loop, tok, other));
            }
        };
    }

    fn send_evt(&self, evt: SessionNotify) {
        let send_res = self.event_sender.send(evt);

        if send_res.is_err() {
            error!("failed to notify event to session: '{:?}'", send_res.err());
        } 
    }

    fn create_socket(&mut self, event_loop: &mut EventLoop, socket_type: SocketType) {
        let id = SocketId(self.id_seq.next());
        let (tx, rx) = mpsc::channel();
        let evt_tx = Rc::new(tx);
        let sig_tx = event_loop.channel();
        let protocol = protocol::create_protocol(id, socket_type, evt_tx.clone());
        let socket = Socket::new(id, protocol, evt_tx.clone(), sig_tx, self.id_seq.clone());

        self.sockets.insert(id, socket);

        self.send_evt(SessionNotify::SocketCreated(id, rx));
    }

    fn destroy_socket(&mut self, event_loop: &mut EventLoop, id: SocketId) {
        if let Some(mut socket) = self.sockets.remove(&id) {
            socket.destroy(event_loop);
        }
    }

    fn create_probe(&mut self, _: &mut EventLoop, left_id: SocketId, right_id: SocketId) {
        if self.sockets.contains_key(&left_id) && self.sockets.contains_key(&right_id) {
            let id = ProbeId(self.id_seq.next());
            let (tx, rx) = mpsc::channel();
            let probe = Probe::new(left_id, right_id, tx);

            self.probes.insert(id, probe);
            self.probe_ids.insert(left_id, id);
            self.probe_ids.insert(right_id, id);

            self.send_evt(SessionNotify::ProbeCreated(id, rx));
        } else {
            let e = invalid_input_io_error("no matching sockets");

            self.send_evt(SessionNotify::ProbeNotCreated(e));
        }

    }

    fn destroy_probe(&mut self, _: &mut EventLoop, id: ProbeId) {
        if let Some(probe) = self.probes.remove(&id) {
            drop(probe);
        }
    }

    fn on_socket_by_id<F>(&mut self, id: &SocketId, action: F) where F : FnOnce(&mut Socket) {
        if let Some(socket) = self.sockets.get_mut(id) {
            action(socket);
        }
    }

    fn on_socket_by_token<F>(&mut self, tok: &mio::Token, action: F) where F : FnOnce(&mut Socket) {
        if let Some(id) = self.socket_ids.get_mut(tok) {
            if let Some(socket) = self.sockets.get_mut(id) {
                action(socket);
            }
        }
    }

    fn reconnect(&mut self, event_loop: &mut EventLoop, tok: mio::Token, addr: String) {
        self.on_socket_by_token(&tok, |s| s.reconnect(addr, event_loop, tok));
    }

    fn rebind(&mut self, event_loop: &mut EventLoop, tok: mio::Token, addr: String) {
        self.on_socket_by_token(&tok, |s| s.rebind(addr, event_loop, tok));
    }

    fn on_send_timeout(&mut self, event_loop: &mut EventLoop, id: SocketId) {
        self.on_socket_by_id(&id, |s| s.on_send_timeout(event_loop));
    }

    fn on_recv_timeout(&mut self, event_loop: &mut EventLoop, id: SocketId) {
        self.on_socket_by_id(&id, |s| s.on_recv_timeout(event_loop));
    }

    fn on_survey_timeout(&mut self, event_loop: &mut EventLoop, id: SocketId) {
        self.on_socket_by_id(&id, |socket| socket.on_survey_timeout(event_loop));
    }

    fn resend(&mut self, event_loop: &mut EventLoop, id: SocketId) {
        self.on_socket_by_id(&id, |socket| socket.resend(event_loop));
    }
}

impl mio::Handler for Session {
    type Timeout = EventLoopTimeout;
    type Message = EventLoopSignal;

    fn notify(&mut self, event_loop: &mut EventLoop, signal: Self::Message) {
        debug!("session received a {} signal", signal.name());

        match signal {
            EventLoopSignal::Cmd(cmd) => self.handle_cmd(event_loop, cmd),
            EventLoopSignal::Evt(evt) => self.handle_evt(event_loop, evt)
        }
    }

    fn ready(&mut self, event_loop: &mut EventLoop, tok: mio::Token, events: mio::EventSet) {
        debug!("ready: [{:?}] '{:?}'", tok.as_usize(), events);

        self.on_socket_by_token(&tok, |socket| socket.ready(event_loop, tok, events));
    }

    fn timeout(&mut self, event_loop: &mut EventLoop, timeout: Self::Timeout) {
        match timeout {
            EventLoopTimeout::Reconnect(token, addr)  => self.reconnect(event_loop, token, addr),
            EventLoopTimeout::Rebind(token, addr)     => self.rebind(event_loop, token, addr),
            EventLoopTimeout::CancelSend(socket_id)   => self.on_send_timeout(event_loop, socket_id),
            EventLoopTimeout::CancelRecv(socket_id)   => self.on_recv_timeout(event_loop, socket_id),
            EventLoopTimeout::CancelSurvey(socket_id) => self.on_survey_timeout(event_loop, socket_id),
            EventLoopTimeout::Resend(socket_id)       => self.resend(event_loop, socket_id)
        }
    }

    fn interrupted(&mut self, _: &mut EventLoop) {

    }
}