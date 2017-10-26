//Implement a simple mutex using bussy waiting. Helpfull source:
//	https://doc.rust-lang.org/core/sync/atomic/
//	https://github.com/rust-lang/rust/blob/master/src/libstd/sync/mutex.rs

use core::sync::atomic::{AtomicBool, Ordering};
use core::ops::{Deref, DerefMut};
use core::cell::UnsafeCell;

//Can be shared across threads
pub struct Spinlock<T: ?Sized>{
	abool: AtomicBool,
	data: UnsafeCell<T>,
}

//invariant: only one instance of this struct exists per Spinlock in state of lock.
//Data is accesed by deref on SpinlockGuard, lock is released automatically when SpinlockGuard is dropped
pub struct SpinlockGuard<'a, T: 'a + ?Sized>{
	spinlock: &'a Spinlock<T>,
}

//If T is send, our spinlock is send and sync
unsafe impl<T: ?Sized + Send> Send for Spinlock<T> { }
unsafe impl<T: ?Sized + Send> Sync for Spinlock<T> { }

impl<'a, T: ?Sized> SpinlockGuard<'a, T>{
	fn new(spinlock: &'a Spinlock<T>) -> Self{
		SpinlockGuard{
			spinlock: spinlock,
		}
	}
}

impl<T> Spinlock<T>{
	pub fn new(t: T) -> Self {
		Spinlock{
			abool: AtomicBool::new(false),
			data: UnsafeCell::new(t),
		}
	}
	pub fn lock(& self) -> SpinlockGuard<T>{
		//replace false with true, if false is safed
		//returns always the previous value
		while self.abool.compare_and_swap(false, true, Ordering::SeqCst){}
		SpinlockGuard::new(self)
	}
}

impl<'a, T: ?Sized> Deref for SpinlockGuard<'a, T>{
	type Target = T;
	
	fn deref(&self) -> &T {
		unsafe { &*self.spinlock.data.get() }
	}
}

impl<'a, T: ?Sized> DerefMut for SpinlockGuard<'a, T>
{
	fn deref_mut (&mut self) -> &mut T {
		unsafe { &mut *self.spinlock.data.get() }
	}
}

impl<'a, T: ?Sized> Drop for SpinlockGuard<'a, T> {
	fn drop(&mut self) {
		self.spinlock.abool.store(false, Ordering::SeqCst);
	}
}