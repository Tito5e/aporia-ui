use std::{
	mem::transmute,
	ptr::{NonNull, addr_of_mut},
};

use crate::reactivity::{NONE, Read, Subscribe, effect::EffectHeader};

#[repr(C)]
pub struct SignalHeader {
	pub subscribers: Vec<NonNull<EffectHeader>>,
}

impl SignalHeader {
	fn notify(&self) {
		for &e in &self.subscribers {
			unsafe {
				let h = e.as_ptr();
				if (*h).token == NONE {
					let s = (*h).sink.as_ptr();
					(*h).token = ((*s).mark)(s, h);
				}
			}
		}
	}
}

#[derive(Clone, Copy)]
pub struct Signal<T> {
	pub(crate) ptr: NonNull<SignalState<T>>,
}

impl<T> Signal<T> {
	pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
		f(unsafe { &(*self.ptr.as_ptr()).value })
	}

	pub fn get(&self) -> &T {
		self.with(|value| unsafe { transmute(value) })
	}

	// TODO: viewの評価中であることをチェック
	pub fn set(&mut self, value: T) {
		let state = unsafe { &mut *self.ptr.as_ptr() };
		state.value = value;
		state.header.notify();
	}

	// TODO: viewの評価中であることをチェック
	pub fn update(&self, f: impl FnOnce(&mut T)) {
		let state = unsafe { &mut *self.ptr.as_ptr() };
		f(&mut state.value);
		state.header.notify();
	}
}

impl<T> Subscribe for Signal<T> {
	fn subscribe(&self, effect: &mut EffectHeader) {
		unsafe {
			let header = addr_of_mut!((*self.ptr.as_ptr()).header);
			(*header).subscribers.push(NonNull::from(&mut *effect));
			effect.deps.push(NonNull::new_unchecked(header));
		}
	}
}

impl<T> Read for Signal<T> {
	type Out<'a>
		= &'a T
	where
		Self: 'a;

	fn read(&self) -> &T {
		self.get()
	}
}

#[repr(C)]
pub(crate) struct SignalState<T> {
	pub(crate) header: SignalHeader,
	pub(crate) value: T,
}
