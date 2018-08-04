// Copyright (c) 2018 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "std")]
use base::error::Error;
use base::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
/// Indicates a failure to allocate memory.
pub struct AllocErr;

impl Display for AllocErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("failed to allocate memory")
    }
}

#[cfg(feature = "std")]
impl Error for AllocErr {}
