use std::ptr::NonNull;

use crate::reactivity::Read;

#[derive(Clone, Copy)]
pub struct ErasedRead<T> {
	data: *const u8,
	read: unsafe fn(*const u8) -> T,
}

impl<T> ErasedRead<T> {
	pub fn new<M>(ptr: NonNull<M>) -> Self
	where
		M: 'static + for<'a> Read<Out<'a> = T>,
	{
		unsafe fn call<M, T>(ptr: *const u8) -> T
		where
			M: 'static + for<'a> Read<Out<'a> = T>,
		{
			unsafe { (*(ptr as *const M)).read() }
		}
		Self { data: ptr.as_ptr() as *const u8, read: call::<M, T> }
	}
}
