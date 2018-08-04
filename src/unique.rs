// Copyright (c) 2018 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

use base::fmt::{self, Debug, Formatter, Pointer};
use base::marker::PhantomData;
use base::ptr::NonNull;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct Unique<T: ?Sized> {
    pointer: NonNull<T>,
    _marker: PhantomData<T>,
}

impl<T: ?Sized> Unique<T> {
    pub(crate) unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self {
            pointer: NonNull::new_unchecked(ptr),
            _marker: PhantomData,
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut T {
        self.pointer.as_ptr()
    }

    pub(crate) unsafe fn as_ref(&self) -> &T {
        &*self.as_ptr()
    }
}

#[cfg(feature = "coerce-unsize")]
impl<T: ?Sized, U: ?Sized> CoerceUnsized<Unique<U>> for Unique<T> where T: Unsize<U> {}

impl<T: ?Sized> Debug for Unique<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Pointer::fmt(&self.as_ptr(), f)
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for Unique<T> {
    fn from(reference: &'a mut T) -> Self {
        Self {
            pointer: NonNull::from(reference),
            _marker: PhantomData,
        }
    }
}

impl<'a, T: ?Sized> From<&'a T> for Unique<T> {
    fn from(reference: &'a T) -> Self {
        Self {
            pointer: NonNull::from(reference),
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> From<NonNull<T>> for Unique<T> {
    fn from(pointer: NonNull<T>) -> Self {
        Self {
            pointer,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Pointer for Unique<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Pointer::fmt(&self.as_ptr(), f)
    }
}

unsafe impl<T: Send + ?Sized> Send for Unique<T> {}

unsafe impl<T: Sync + ?Sized> Sync for Unique<T> {}
