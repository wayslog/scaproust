// Copyright 2016 Benoît Labaere (benoit.labaere@gmail.com)
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>
// or the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt;
use std::rc::Rc;
use std::cell::Cell;
use std::io::{ Error, ErrorKind };
use std::time;

use mio::NotifyError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SocketType {
    Pair       = (1 * 16),
    Pub        = (2 * 16),
    Sub        = (2 * 16) + 1,
    Req        = (3 * 16),
    Rep        = (3 * 16) + 1,
    Push       = (5 * 16),
    Pull       = (5 * 16) + 1,
    Surveyor   = (6 * 16) + 2,
    Respondent = (6 * 16) + 3,
    Bus        = (7 * 16)
}

impl SocketType {
    pub fn id(&self) -> u16 {
        *self as u16
    }

    pub fn peer(&self) -> SocketType {
        match *self {
            SocketType::Pair       => SocketType::Pair,
            SocketType::Pub        => SocketType::Sub,
            SocketType::Sub        => SocketType::Pub,
            SocketType::Req        => SocketType::Rep,
            SocketType::Rep        => SocketType::Req,
            SocketType::Push       => SocketType::Pull,
            SocketType::Pull       => SocketType::Push,
            SocketType::Surveyor   => SocketType::Respondent,
            SocketType::Respondent => SocketType::Surveyor,
            SocketType::Bus        => SocketType::Bus
        }
    }

    pub fn matches(&self, other: SocketType) -> bool {
        self.peer() == other &&
        other.peer() == *self
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct SocketId(pub usize);

impl fmt::Debug for SocketId {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ProbeId(pub usize);

impl fmt::Debug for ProbeId {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone)]
pub struct IdSequence {
    value: Rc<Cell<usize>>
}

impl IdSequence {
    pub fn new() -> IdSequence {
        IdSequence { value: Rc::new(Cell::new(0)) }
    }

    pub fn next(&self) -> usize {
        let id = self.value.get();

        self.value.set(id + 1);
        id
    }
}

pub fn other_io_error(msg: &'static str) -> Error {
    Error::new(ErrorKind::Other, msg)
}

pub fn invalid_data_io_error(msg: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, msg)
}

pub fn would_block_io_error(msg: &'static str) -> Error {
    Error::new(ErrorKind::WouldBlock, msg)
}

pub fn invalid_input_io_error(msg: &'static str) -> Error {
    Error::new(ErrorKind::InvalidInput, msg)
}

pub fn convert_notify_err<T>(err: NotifyError<T>) -> Error {
    match err {
        NotifyError::Io(e)     => e,
        NotifyError::Closed(_) => other_io_error("cmd channel closed"),
        NotifyError::Full(_)   => Error::new(ErrorKind::WouldBlock, "cmd channel full")
    }
}

pub trait ToMillis {
    fn to_millis(&self) -> u64;
}

impl ToMillis for time::Duration {
    fn to_millis(&self) -> u64 {
        let millis_from_secs = self.as_secs() * 1_000;
        let millis_from_nanos = self.subsec_nanos() as f64 / 1_000_000f64;

        millis_from_secs + millis_from_nanos as u64
    }
}

#[cfg(test)]
mod tests {
    use super::IdSequence;

    #[test]
    fn id_sequence_can_be_cloned() {
        let seq = IdSequence::new();
        let other = seq.clone();

        assert_eq!(0, other.next());
        assert_eq!(1, seq.next());
        assert_eq!(2, seq.next());
        assert_eq!(3, other.next());
    }
}