use std::{
	ops::{Deref, DerefMut},
	ptr::{NonNull, addr_of_mut},
};

use crate::reactivity::{NONE, signal::SignalHeader, sink::SinkHeader};

#[repr(C)]
pub struct EffectHeader {
	pub sink: NonNull<SinkHeader>,
	pub token: u32,
	pub deps: Vec<NonNull<SignalHeader>>,
}

impl EffectHeader {
	pub fn begin_run(&mut self) {
		self.token = NONE;
	}

	pub fn unsubscribe_all(&mut self) {
		let self_ptr = self as *mut EffectHeader;
		for dep in self.deps.drain(..) {
			unsafe {
				let subs = &mut (*dep.as_ptr()).subscribers;
				if let Some(index) = subs.iter().position(|effect| effect.as_ptr() == self_ptr) {
					subs.swap_remove(index);
				}
			}
		}
	}
}

#[repr(C)]
pub struct EffectState<F> {
	pub header: EffectHeader,
	pub f: F,
}

#[derive(Clone, Copy)]
pub struct EffectHandle<F> {
	pub(crate) ptr: NonNull<EffectState<F>>,
}

impl<F> Deref for EffectHandle<F> {
	type Target = EffectHeader;
	fn deref(&self) -> &EffectHeader {
		unsafe { &(*self.ptr.as_ptr()).header }
	}
}

impl<F> DerefMut for EffectHandle<F> {
	fn deref_mut(&mut self) -> &mut EffectHeader {
		unsafe { &mut (*self.ptr.as_ptr()).header }
	}
}

impl<F> EffectHandle<F> {
	pub fn invalidate(&self) {
		unsafe {
			let header = addr_of_mut!((*self.ptr.as_ptr()).header);
			if (*header).token == NONE {
				let sink = (*header).sink.as_ptr();
				(*header).token = ((*sink).mark)(sink, header);
			}
		}
	}
}
