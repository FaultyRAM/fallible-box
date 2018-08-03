// Copyright (c) 2018 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides a `Box` type that returns an error on OOM instead of panicking.

#![forbid(warnings)]
#![forbid(future_incompatible)]
#![forbid(unused)]
#![forbid(missing_copy_implementations)]
#![forbid(missing_debug_implementations)]
#![forbid(missing_docs)]
#![forbid(trivial_casts)]
#![forbid(trivial_numeric_casts)]
#![forbid(unused_qualifications)]
#![forbid(unused_results)]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_complexity))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_correctness))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_perf))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_style))]
