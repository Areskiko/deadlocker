#[cfg(feature = "derive")]
extern crate deadlocker_derive;

#[cfg(feature = "derive")]
pub use deadlocker_derive::Locker;

pub trait Locker<'a> {
    type LockBuilder
    where
        Self: 'a;
    fn locker(&'a mut self) -> Self::LockBuilder;
}
