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
#![cfg_attr(feature = "exact-size-is-empty", feature(exact_size_is_empty))]
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
#![cfg_attr(feature = "cargo-clippy", allow(partialeq_ne_impl))]
#![cfg_attr(feature = "cargo-clippy", allow(use_self))]
#![cfg_attr(feature = "cargo-clippy", allow(wrong_self_convention))]

#[cfg(not(feature = "std"))]
extern crate alloc as alloc_crate;

#[cfg(not(feature = "std"))]
mod base {
    pub use alloc_crate::alloc;
    pub use core::{any, cmp, fmt, hash, hint, iter, marker, mem, ops, ptr};
}
#[cfg(feature = "std")]
mod base {
    pub use std::{alloc, any, cmp, error, fmt, hash, hint, iter, marker, mem, ops, ptr};
}

mod alloc_err;

pub use alloc_err::AllocErr;

use base::alloc::{self, Layout};
use base::any::Any;
use base::cmp::Ordering;
use base::fmt::{self, Debug, Display, Formatter, Pointer};
use base::hash::{Hash, Hasher};
use base::iter::FusedIterator;
use base::marker::PhantomData;
#[cfg(feature = "coerce-unsized")]
use base::marker::Unsize;
#[cfg(feature = "coerce-unsized")]
use base::ops::CoerceUnsized;
use base::ops::{Deref, DerefMut};
use base::ptr::{self, NonNull};
use base::{hint, mem};

#[inline]
fn layout_of<T>() -> Layout {
    unsafe {
        Layout::from_size_align(mem::size_of::<T>(), mem::align_of::<T>())
            .unwrap_or_else(|_| hint::unreachable_unchecked())
    }
}

#[inline]
fn layout_of_val<T: ?Sized>(x: &T) -> Layout {
    unsafe {
        Layout::from_size_align(mem::size_of_val(x), mem::align_of_val(x))
            .unwrap_or_else(|_| hint::unreachable_unchecked())
    }
}

#[inline]
fn alloc_memory<T>() -> Result<NonNull<T>, AllocErr> {
    unsafe {
        let p = alloc::alloc(layout_of::<T>()) as *mut T;
        if p.is_null() {
            Err(AllocErr)
        } else {
            Ok(NonNull::new_unchecked(p))
        }
    }
}

#[inline]
fn free_memory<T: ?Sized>(p: NonNull<T>) {
    unsafe { alloc::dealloc(p.as_ptr() as *mut u8, layout_of_val(p.as_ref())) }
}

/// A smart pointer type that safely manages heap memory.
pub struct Box<T: ?Sized> {
    pointer: NonNull<T>,
    marker: PhantomData<T>,
}

impl<T> Box<T> {
    #[inline]
    /// Tries to allocate memory on the heap, and moves `x` into it if successful.
    pub fn try_new(x: T) -> Result<Self, AllocErr> {
        unsafe {
            alloc_memory().map(|pointer| {
                ptr::write(pointer.as_ptr(), x);
                Self {
                    pointer,
                    marker: PhantomData,
                }
            })
        }
    }
}

impl<T: ?Sized> Box<T> {
    #[inline]
    /// Creates a box from a non-null pointer.
    ///
    /// This is `unsafe` because there is no guarantee that the given raw pointer is valid. In
    /// other words, calling this function with any raw pointer other than one returned from
    /// `Box::into_raw` can result in undefined behaviour.
    pub unsafe fn from_non_null(pointer: NonNull<T>) -> Self {
        Self {
            pointer,
            marker: PhantomData,
        }
    }

    #[inline]
    /// Consumes a box, returning the wrapped pointer without freeing the memory it refers to.
    ///
    /// After calling this function, the caller is responsible for ensuring that the memory
    /// referenced by the returned pointer is correctly freed. This can be done by passing the
    /// pointer to `Box::from_raw` and dropping the returned box.
    pub fn into_non_null(b: Self) -> NonNull<T> {
        let p = b.pointer;
        mem::forget(b);
        p
    }

    #[inline]
    /// Consumes a box, leaking its contents.
    pub fn leak<'a>(b: Self) -> &'a mut T {
        unsafe {
            let p = b.pointer;
            mem::forget(b);
            &mut *p.as_ptr()
        }
    }
}

impl Box<dyn Any> {
    /// Tries to downcast a box into a discrete type without allocating, returning the original box
    /// on failure.
    pub fn downcast<T: Any>(self) -> Result<Box<T>, Self> {
        unsafe {
            if self.is::<T>() {
                let p = Self::into_non_null(self).as_ptr() as *mut T;
                Ok(Box::<T>::from_non_null(NonNull::new_unchecked(p)))
            } else {
                Err(self)
            }
        }
    }
}

impl<T: Clone> Box<T> {
    #[inline]
    /// Tries to clone a box.
    pub fn try_clone(&self) -> Result<Self, AllocErr> {
        Self::try_new(self.as_ref().clone())
    }

    #[inline]
    /// Performs copy assignment into one box from another, without allocating.
    pub fn clone_from(&mut self, other: &Self) {
        unsafe {
            let c = other.as_ref().clone();
            ptr::drop_in_place(self.pointer.as_ptr());
            ptr::write(self.pointer.as_ptr(), c);
        }
    }
}

impl<T: Default> Box<T> {
    /// Tries to default-construct a boxed instance of a particular type.
    pub fn try_default() -> Result<Self, AllocErr> {
        Self::try_new(T::default())
    }
}

impl<T: ?Sized> AsMut<T> for Box<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { self.pointer.as_mut() }
    }
}

impl<T: ?Sized> AsRef<T> for Box<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.pointer.as_ref() }
    }
}

#[cfg(feature = "coerce-unsized")]
impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Box<U>> for Box<T> {}

impl<T: ?Sized + Debug> Debug for Box<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T: ?Sized> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: ?Sized> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T: ?Sized + Display> Display for Box<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T: ?Sized + DoubleEndedIterator> DoubleEndedIterator for Box<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.as_mut().next_back()
    }
}

impl<T: ?Sized> Drop for Box<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.pointer.as_ptr());
            free_memory(self.pointer);
        }
    }
}

impl<T: ?Sized + Eq> Eq for Box<T> {}

impl<T: ?Sized + ExactSizeIterator> ExactSizeIterator for Box<T> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }

    #[cfg(feature = "exact-size-is-empty")]
    fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }
}

impl<T: ?Sized + FusedIterator> FusedIterator for Box<T> {}

impl<T: ?Sized + Hash> Hash for Box<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl<T: ?Sized + Hasher> Hasher for Box<T> {
    fn finish(&self) -> u64 {
        self.as_ref().finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.as_mut().write(bytes)
    }

    fn write_u8(&mut self, i: u8) {
        self.as_mut().write_u8(i)
    }

    fn write_u16(&mut self, i: u16) {
        self.as_mut().write_u16(i)
    }

    fn write_u32(&mut self, i: u32) {
        self.as_mut().write_u32(i)
    }

    fn write_u64(&mut self, i: u64) {
        self.as_mut().write_u64(i)
    }

    fn write_u128(&mut self, i: u128) {
        self.as_mut().write_u128(i)
    }

    fn write_usize(&mut self, i: usize) {
        self.as_mut().write_usize(i)
    }

    fn write_i8(&mut self, i: i8) {
        self.as_mut().write_i8(i)
    }

    fn write_i16(&mut self, i: i16) {
        self.as_mut().write_i16(i)
    }

    fn write_i32(&mut self, i: i32) {
        self.as_mut().write_i32(i)
    }

    fn write_i64(&mut self, i: i64) {
        self.as_mut().write_i64(i)
    }

    fn write_i128(&mut self, i: i128) {
        self.as_mut().write_i128(i)
    }

    fn write_isize(&mut self, i: isize) {
        self.as_mut().write_isize(i)
    }
}

impl<T: ?Sized + Iterator> Iterator for Box<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.as_mut().next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.as_ref().size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.as_mut().nth(n)
    }
}

impl<T: ?Sized + Ord> Ord for Box<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other)
    }
}

impl<T: ?Sized + PartialEq> PartialEq for Box<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other)
    }

    #[inline]
    fn ne(&self, other: &Self) -> bool {
        self.as_ref().ne(other)
    }
}

impl<T: ?Sized + PartialOrd> PartialOrd for Box<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other)
    }

    #[inline]
    fn lt(&self, other: &Self) -> bool {
        self.as_ref().lt(other)
    }

    #[inline]
    fn le(&self, other: &Self) -> bool {
        self.as_ref().le(other)
    }

    #[inline]
    fn gt(&self, other: &Self) -> bool {
        self.as_ref().gt(other)
    }

    #[inline]
    fn ge(&self, other: &Self) -> bool {
        self.as_ref().ge(other)
    }
}

impl<T: ?Sized> Pointer for Box<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let p: *const T = self.pointer.as_ptr();
        Pointer::fmt(&p, f)
    }
}

unsafe impl<T: ?Sized + Send> Send for Box<T> {}

unsafe impl<T: ?Sized + Sync> Sync for Box<T> {}

#[cfg(test)]
mod tests {
    use Box;

    #[test]
    fn alloc_zst() {
        let a = Box::try_new(()).unwrap();
        assert_eq!(*a, ());
    }

    #[test]
    fn alloc_bool() {
        let _ = Box::try_new(true).unwrap();
    }

    #[test]
    fn alloc_u8() {
        let _ = Box::try_new(1_u8).unwrap();
    }

    #[test]
    fn alloc_u64() {
        let _ = Box::try_new(4_u64).unwrap();
    }

    #[test]
    fn alloc_array() {
        let _ = Box::try_new([1_u8; 256]).unwrap();
    }

    #[test]
    fn alloc_struct() {
        struct Foo(u64, u32);
        let _ = Box::try_new(Foo(1, 2)).unwrap();
    }

    #[test]
    fn equality() {
        let a = Box::try_new(5).unwrap();
        let b = Box::try_new(5).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn inequality() {
        let a = Box::try_new(5).unwrap();
        let b = Box::try_new(6).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn lt() {
        let a = Box::try_new(5).unwrap();
        let b = Box::try_new(6).unwrap();
        assert!(a < b);
    }

    #[test]
    fn le() {
        let a = Box::try_new(5).unwrap();
        let b = Box::try_new(5).unwrap();
        let c = Box::try_new(6).unwrap();
        assert!(a <= b);
        assert!(a <= c);
    }

    #[test]
    fn gt() {
        let a = Box::try_new(6).unwrap();
        let b = Box::try_new(5).unwrap();
        assert!(a > b);
    }

    #[test]
    fn ge() {
        let a = Box::try_new(6).unwrap();
        let b = Box::try_new(6).unwrap();
        let c = Box::try_new(5).unwrap();
        assert!(a >= b);
        assert!(a >= c);
    }

    #[test]
    fn clone() {
        let a = Box::try_new(5).unwrap();
        let b = a.try_clone().unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn default() {
        let a = Box::<u16>::try_default().unwrap();
        let b = u16::default();
        assert_eq!(*a, b);
    }
}
