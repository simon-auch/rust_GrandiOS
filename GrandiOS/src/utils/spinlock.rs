//Implement a simple mutex using bussy waiting. Helpfull source:
//	https://doc.rust-lang.org/core/sync/atomic/
//	https://github.com/rust-lang/rust/blob/master/src/libstd/sync/mutex.rs
//	http://embed.rs/articles/2016/arm-inline-assembly-rust/

use core::ops::{Deref, DerefMut};
use core::cell::UnsafeCell;

//function used for syncronisation
//atomically moves new into tmp and the old value from tmp is returned
fn swap(new: & u8, tmp: & u8) -> u8{
	let mut ret : u8 = 0;
	unsafe {
		asm!(
			"SWPB r0, r1, [r2]" :
			"={r0}"(ret):/*outputs*/
			"{r1}"(new), "{r2}"(tmp as *const u8):/*inputs*/
			:/*clobbers*/
			"volatile"/*options*/
		);
	}
	return ret;
}


//Can be shared across threads
pub struct Spinlock<T: ?Sized>{
	val: u8,
	data: UnsafeCell<T>,
}

//invariant: only one instance of this struct exists per Spinlock in state of lock.
//Data is accesed by deref on SpinlockGuard, lock is released automatically when SpinlockGuard is dropped
pub struct SpinlockGuard<'a, T: 'a>{
	spinlock: &'a Spinlock<T>,
}

//If T is send, our spinlock is send and sync
unsafe impl<T: Send> Send for Spinlock<T> { }
unsafe impl<T: Send> Sync for Spinlock<T> { }

impl<'a, T> SpinlockGuard<'a, T>{
	fn new(spinlock: &'a Spinlock<T>) -> Self{
		SpinlockGuard{
			spinlock: spinlock,
		}
	}
}

impl<T> Spinlock<T>{
	pub fn new(t: T) -> Self {
		Spinlock{
			val: 0,
			data: UnsafeCell::new(t),
		}
	}
	pub fn lock(& self) -> SpinlockGuard<T>{
		//replace false with true, if false is safed
		//returns always the previous value
		let mut val: u8 = 1;
		while val==1{
			val = swap(& val,& self.val);
		}
		SpinlockGuard::new(self)
	}
	fn unlock(& self){
		let val: u8 = 0;
		swap(& val,& self.val);
	}
}

impl<'a, T> Deref for SpinlockGuard<'a, T>{
	type Target = T;
	
	fn deref(&self) -> &T {
		unsafe { &*self.spinlock.data.get() }
	}
}

impl<'a, T> DerefMut for SpinlockGuard<'a, T>
{
	fn deref_mut (&mut self) -> &mut T {
		unsafe { &mut *self.spinlock.data.get() }
	}
}

impl<'a, T> Drop for SpinlockGuard<'a, T> {
	fn drop(&mut self) {
		self.spinlock.unlock();
	}
}