// Copyright 2016 Benoît Labaere (benoit.labaere@gmail.com)
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>
// or the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// This file may not be copied, modified, or distributed except according to those terms.

pub mod pair;
pub mod push;
pub mod pull;
pub mod publ;

mod priolist;

use core::context::Scheduled;

pub const PAIR: u16 = (1 * 16);
pub const PUB:  u16 = (2 * 16);
pub const SUB:  u16 = (2 * 16) + 1;
pub const PUSH: u16 = (5 * 16);
pub const PULL: u16 = (5 * 16) + 1;

pub type Timeout = Option<Scheduled>;
