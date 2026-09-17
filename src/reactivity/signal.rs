use crate::reactivity::{context::Context, subscriber::Subscriber};
use std::{collections::HashSet, ffi::c_void, marker::PhantomData};

/// Internal State for Signal
pub(crate) struct SignalState {
	pub(crate) value: SignalValue,
	pub(crate) subscribers: HashSet<Subscriber>,
}

impl SignalState {
	pub unsafe fn read<T>(&self) -> &T {
		unsafe { self.value.read() }
	}

	pub unsafe fn write<T>(&mut self, value: T) {
		unsafe {
			self.value.write(value);
		}
	}

	pub fn subscribe(&mut self, subscriber: Subscriber) {
		self.subscribers.insert(subscriber);
	}
}

/// Type Erased Value
pub(crate) struct SignalValue {
	ptr: *mut c_void,
}

impl SignalValue {
	pub fn new(ptr: *mut c_void) -> Self {
		Self { ptr }
	}

	pub unsafe fn read<T>(&self) -> &T {
		let typed_ptr = self.ptr as *const T;

		unsafe { &*typed_ptr }
	}

	pub unsafe fn write<T>(&mut self, value: T) {
		let value_ref = unsafe { &mut *(self.ptr as *mut T) };
		*value_ref = value;
	}
}

pub struct Signal<T> {
	pub(crate) state_ptr: *mut SignalState,
	pub(crate) phantom: PhantomData<T>,
}

impl<T> Signal<T> {
	pub fn new(initial_value: T) -> Self {
		let signal = Context::create_signal(initial_value);

		signal
	}

	pub(crate) fn get_untracked(&self) -> &T {
		unsafe {
			let state = &*self.state_ptr;
			state.read()
		}
	}

	pub fn get(&self) -> &T {
		unsafe {
			let state = &mut *self.state_ptr;

			let current_mounter = Context::get_current_mounter();
			if let Some(current_mounter) = current_mounter {
				state.subscribe(current_mounter);
			} else {
				panic!("Dont read signal value outside Component")
			}
			state.read()
		}
	}

	pub fn set(&self, value: T) {
		unsafe {
			let state = &mut *self.state_ptr;
			state.write(value);
		}
	}
}
