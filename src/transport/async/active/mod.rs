// Copyright 2016 Benoît Labaere (benoit.labaere@gmail.com)
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>
// or the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// This file may not be copied, modified, or distributed except according to those terms.

#[cfg(unix)] pub use self::unix::Active;
#[cfg(unix)] mod unix;

#[cfg(windows)] pub use self::windows::Active;
#[cfg(windows)] mod windows;
