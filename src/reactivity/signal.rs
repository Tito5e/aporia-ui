use slotmap::new_key_type;

use crate::reactivity::{
	context::{CONTEXT, Context},
	effect::Effect,
};
use std::{collections::HashSet, ffi::c_void, marker::PhantomData, mem::transmute};

new_key_type! { pub(crate) struct SignalKey; }

/// Internal State for Signal
pub(crate) struct SignalState {
	pub(crate) value: SignalValue,
	pub(crate) subscribers: HashSet<Effect>,
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

	pub fn subscribe(&mut self, subscriber: Effect) {
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
	pub(crate) state_key: SignalKey,
	pub(crate) phantom: PhantomData<T>,
}

impl<T> Signal<T> {
	pub fn new(initial_value: T) -> Self {
		let signal = Context::create_signal(initial_value);

		signal
	}

	pub(crate) fn get_untracked(&self) -> &T {
		unsafe {
			let state: &mut SignalState = CONTEXT.with(|context| {
				transmute(context.signals.borrow_mut().get_unchecked_mut(self.state_key)
					as &mut SignalState)
			});

			state.read()
		}
	}

	pub fn get(&self) -> &T {
		unsafe {
			let state: &mut SignalState = CONTEXT.with(|context| {
				transmute(context.signals.borrow_mut().get_unchecked_mut(self.state_key)
					as &mut SignalState)
			});

			let current_scope = Context::get_current_effect();
			if let Some(current_scope) = current_scope {
				state.subscribe(Effect::Build(current_scope));
			} else {
				panic!("Dont read signal value outside Component")
			}
			state.read()
		}
	}

	pub fn set(&self, value: T) {
		unsafe {
			let state: &mut SignalState = CONTEXT.with(|context| {
				transmute(context.signals.borrow_mut().get_unchecked_mut(self.state_key)
					as &mut SignalState)
			});
			state.write(value);

			state.subscribers.iter().for_each(|effect| effect.invalidate());
		}
	}

	pub(crate) fn subscribe(&self, subscriber: Effect) {
		unsafe {
			let state: &mut SignalState = CONTEXT.with(|context| {
				transmute(context.signals.borrow_mut().get_unchecked_mut(self.state_key)
					as &mut SignalState)
			});
			state.subscribe(subscriber);
		}
	}
}
