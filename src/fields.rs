// Copyright 2025 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Wrapper types for MMIO fields.

use core::{
    cell::UnsafeCell,
    fmt::{self, Debug, Formatter},
};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Wrapper for a field which may safely be read but not written. Reading may cause side-effects,
/// changing the state of the device in some way.
#[derive(Clone, Debug, Default, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct ReadOnly<T>(pub T);

/// Wrapper for a field which may safely be read with no side-effects but not written.
#[derive(Clone, Debug, Default, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct ReadPure<T>(pub T);

/// Wrapper for a field which may safely be written but not read.
#[derive(Clone, Debug, Default, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct WriteOnly<T>(pub T);

/// Wrapper for a field which may safely be written and read. Reading may cause side-effects,
/// changing the state of the device in some way.
#[derive(Clone, Debug, Default, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct ReadWrite<T>(pub T);

/// Wrapper for a field which may safely be written (with side-effects) and read with no
/// side-effects.
#[derive(Clone, Debug, Default, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct ReadPureWrite<T>(pub T);

/// Wrapper for a field which may safely be written from shared pointers, but not read.
///
/// Multiple threads or cores may write to this field concurrently (e.g. write-only doorbell or
/// software-generated interrupt registers such as `GICD_SGIR` or `GICD_SETSPI_NSR`).
#[derive(KnownLayout)]
#[repr(transparent)]
pub struct WriteShared<T>(UnsafeCell<T>);

// SAFETY:
// - Direct `&WriteShared<T>` references never read or write `self.0`: the inner `UnsafeCell<T>`
//   field is private, and `Debug::fmt` does not access `self.0`.
// - Because `WriteShared<T>` has a private field and does not implement `Default`, `Clone`,
//   `From`, `FromZeros`, or `FromBytes`, safe code cannot construct inhabited values of
//   `WriteShared<T>` in ordinary memory to pass to `SharedMmioPointer::from` or
//   `UniqueMmioPointer::from`. Any `SharedMmioPointer` or `UniqueMmioPointer` targeting a
//   `WriteShared<T>` must therefore originate from `UniqueMmioPointer::new` (or `child` /
//   `split_child`), whose `# Safety` precondition guarantees that concurrent MMIO writes from any
//   thread are valid, and `T: Send + Sync` permits transferring values of `T` across threads.
unsafe impl<T: Send + Sync> Sync for WriteShared<T> {}

impl<T> Debug for WriteShared<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("WriteShared").finish_non_exhaustive()
    }
}

/// Wrapper for a field which may safely be read (with side-effects) from a unique pointer and
/// safely written from shared pointers.
///
/// For example, a UART data register where reading pops from the receive FIFO (requiring exclusive
/// access) while writing pushes to the transmit FIFO (safe to do concurrently, such as from a
/// lock-free panic handler).
#[derive(KnownLayout)]
#[repr(transparent)]
pub struct ReadWriteShared<T>(UnsafeCell<T>);

// SAFETY:
// - Direct `&ReadWriteShared<T>` references never read or write `self.0`: the inner
//   `UnsafeCell<T>` field is private, and `Debug::fmt` does not access `self.0`.
// - Because `ReadWriteShared<T>` has a private field and does not implement `Default`, `Clone`,
//   `From`, `FromZeros`, or `FromBytes`, safe code cannot construct inhabited values of
//   `ReadWriteShared<T>` in ordinary memory to pass to `SharedMmioPointer::from` or
//   `UniqueMmioPointer::from`. Any `SharedMmioPointer` or `UniqueMmioPointer` targeting a
//   `ReadWriteShared<T>` must therefore originate from `UniqueMmioPointer::new` (or `child` /
//   `split_child`), whose `# Safety` precondition guarantees that concurrent MMIO writes from any
//   thread (including alongside exclusive reads) are valid, and `T: Send + Sync` permits
//   transferring values of `T` across threads.
unsafe impl<T: Send + Sync> Sync for ReadWriteShared<T> {}

impl<T> Debug for ReadWriteShared<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadWriteShared").finish_non_exhaustive()
    }
}

/// Wrapper for a field which may safely be read with no side-effects and safely written from
/// shared pointers.
///
/// Multiple threads or cores may read and write this field concurrently (e.g. write-1-to-set or
/// write-1-to-clear registers such as `GICD_ISENABLERn` and `GICD_ICENABLERn`).
///
/// Values of this type cannot be constructed in safe code, preventing safe callers from creating
/// a writable [`SharedMmioPointer`](crate::SharedMmioPointer) from a shared reference to ordinary
/// memory:
///
/// ```compile_fail
/// use safe_mmio::{SharedMmioPointer, fields::ReadPureWriteShared};
///
/// let fake = [ReadPureWriteShared(0u32)];
/// let shared = SharedMmioPointer::from(fake.as_slice());
/// shared.get(0).unwrap().write(42);
/// ```
#[derive(KnownLayout)]
#[repr(transparent)]
pub struct ReadPureWriteShared<T>(UnsafeCell<T>);

// SAFETY:
// - Direct `&ReadPureWriteShared<T>` references never read or write `self.0`: the inner
//   `UnsafeCell<T>` field is private, and `Debug::fmt` does not access `self.0`.
// - Because `ReadPureWriteShared<T>` has a private field and does not implement `Default`,
//   `Clone`, `From`, `FromZeros`, or `FromBytes`, safe code cannot construct inhabited values of
//   `ReadPureWriteShared<T>` in ordinary memory to pass to `SharedMmioPointer::from` or
//   `UniqueMmioPointer::from`. Any `SharedMmioPointer` or `UniqueMmioPointer` targeting a
//   `ReadPureWriteShared<T>` must therefore originate from `UniqueMmioPointer::new` (or `child` /
//   `split_child`), whose `# Safety` precondition guarantees that concurrent MMIO reads and writes
//   from any thread are valid, and `T: Send + Sync` permits transferring values of `T` across
//   threads.
unsafe impl<T: Send + Sync> Sync for ReadPureWriteShared<T> {}

impl<T> Debug for ReadPureWriteShared<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadPureWriteShared")
            .finish_non_exhaustive()
    }
}
