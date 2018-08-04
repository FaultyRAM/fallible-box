// Copyright (c) 2018 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides a `Box` type that returns an error on OOM instead of panicking.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]
#![cfg_attr(feature = "coerce-unsized", feature(coerce_unsized))]
#![cfg_attr(feature = "coerce-unsized", feature(unsize))]
#![deny(warnings)]
#![forbid(future_incompatible)]
#![forbid(unused)]
#![forbid(missing_debug_implementations)]
#![forbid(missing_docs)]
#![forbid(trivial_casts)]
#![forbid(trivial_numeric_casts)]
#![deny(unused_qualifications)]
#![forbid(unused_results)]
#![allow(intra_doc_link_resolution_failure)]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_complexity))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_correctness))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_perf))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_style))]

#[cfg(not(feature = "std"))]
extern crate alloc as alloc_crate;

#[cfg(not(feature = "std"))]
mod base {
    pub use alloc_crate::alloc;
    pub use core::{fmt, hint, marker, mem, ops, ptr};
}
#[cfg(feature = "std")]
mod base {
    pub use std::{alloc, error, fmt, hint, marker, mem, ops, ptr};
}

mod alloc_err;
mod unique;

pub use alloc_err::AllocErr;

use base::alloc::{self, Layout};
use base::fmt::{self, Debug, Formatter, Pointer};
use base::ptr::{self, NonNull};
use base::{hint, mem};
use unique::Unique;

/// A smart pointer type that safely manages heap memory.
pub struct Box<T: ?Sized>(Unique<T>);

impl<T> Box<T> {
    #[inline]
    /// Allocates memory on the heap and moves `x` into it.
    pub fn new(x: T) -> Result<Self, AllocErr> {
        unsafe {
            let p = alloc::alloc(Self::get_layout(&x)) as *mut T;
            if p.is_null() {
                Err(AllocErr)
            } else {
                ptr::write(p, x);
                Ok(Box(Unique::new_unchecked(p)))
            }
        }
    }
}

impl<T: ?Sized> Box<T> {
    #[inline]
    /// Creates a box from a non-null raw pointer.
    ///
    /// This is `unsafe` because there is no guarantee that the given raw pointer is valid. In
    /// other words, calling this function with any raw pointer other than one returned from
    /// `Box::into_raw` can result in undefined behaviour.
    pub unsafe fn from_raw(x: NonNull<T>) -> Self {
        Box(Unique::from(x))
    }

    #[inline]
    fn get_layout(x: &T) -> Layout {
        unsafe {
            Layout::from_size_align(mem::size_of_val(x), mem::align_of_val(x))
                .unwrap_or_else(|_| hint::unreachable_unchecked())
        }
    }
}

impl<T: ?Sized> AsRef<T> for Box<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}

impl<T: Debug + ?Sized> Debug for Box<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(self.as_ref(), f)
    }
}

impl<T: ?Sized> Drop for Box<T> {
    fn drop(&mut self) {
        unsafe {
            alloc::dealloc(self.0.as_ptr() as *mut u8, Self::get_layout(self.as_ref()));
        }
    }
}

impl<T: ?Sized> Pointer for Box<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let p: *const T = self.0.as_ptr();
        Pointer::fmt(&p, f)
    }
}
