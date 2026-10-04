use std::{
	mem::transmute,
	ptr::{NonNull, addr_of_mut},
};

use crate::reactivity::{NONE, Source, effect::EffectHeader};

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
	pub(crate) fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
		f(unsafe { &(*self.ptr.as_ptr()).value })
	}

	pub(crate) unsafe fn get(&self) -> &T {
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

impl<T: Clone> Source for Signal<T> {
	type Out = T;

	fn read(&self) -> T {
		unsafe { self.get().clone() }
	}

	fn subscribe(&self, effect: &mut EffectHeader) -> T {
		unsafe {
			let header = NonNull::new_unchecked(addr_of_mut!((*self.ptr.as_ptr()).header));
			if !effect.deps.contains(&header) {
				(*header.as_ptr()).subscribers.push(NonNull::from(&mut *effect));
				effect.deps.push(header);
			}

			(*self.ptr.as_ptr()).value.clone()
		}
	}
}

#[repr(C)]
pub(crate) struct SignalState<T> {
	pub(crate) header: SignalHeader,
	pub(crate) value: T,
}
