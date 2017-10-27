//Implement a simple mutex using bussy waiting. Helpfull source:
//	https://doc.rust-lang.org/core/sync/atomic/
//	https://github.com/rust-lang/rust/blob/master/src/libstd/sync/mutex.rs
//	http://embed.rs/articles/2016/arm-inline-assembly-rust/

use core::ops::{Deref, DerefMut};
use core::cell::UnsafeCell;

//function used for syncronisation
//atomically moves new into tmp and the old value from tmp is returned
fn swap(val1: &mut u8, val2: &mut u8){
	unsafe {
		asm!(
			"SWPB $0, $0, [$1]" :
			"=r"(*val1):/*outputs*/
			"r"(val2):/*inputs*/
			:/*clobbers*/
			"volatile"/*options*/
		);
	}
}


//Can be shared across threads
pub struct Spinlock<T: ?Sized>{
	val: UnsafeCell<u8>,
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
			val: UnsafeCell::new(0),
			data: UnsafeCell::new(t),
		}
	}
	pub fn lock(& self) -> SpinlockGuard<T>{
		//replace false with true, if false is safed
		//returns always the previous value
		let mut val: u8 = 1;
		while val==1{
			swap(&mut val, unsafe { &mut *self.val.get() } );
		}
		SpinlockGuard::new(self)
	}
	fn unlock(& self){
		let mut val: u8 = 0;
		swap(&mut val, unsafe { &mut *self.val.get() });
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
