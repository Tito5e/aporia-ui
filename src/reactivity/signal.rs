use slotmap::new_key_type;

use crate::{
	reactivity::{
		context::{CONTEXT, Context},
		effect::Effect,
	},
	storage::Key,
};
use std::{collections::HashSet, marker::PhantomData, mem::transmute};

new_key_type! { pub(crate) struct SignalKey; }

/// Internal State for Signal
pub(crate) struct SignalState {
	pub(crate) value: SignalValue,
	pub(crate) subscribers: HashSet<Effect>,
}

impl SignalState {
	pub unsafe fn read<T: 'static>(&mut self) -> &T {
		unsafe { self.value.read() }
	}

	pub unsafe fn write<T: 'static>(&mut self, value: T) {
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
	key: Key,
}

impl SignalValue {
	pub fn new(key: Key) -> Self {
		Self { key }
	}

	pub unsafe fn read<T: 'static>(&mut self) -> &T {
		unsafe { Context::read_signal::<T>(&mut self.key) }
	}

	pub unsafe fn write<T: 'static>(&mut self, value: T) {
		let value_ref = unsafe { Context::read_signal::<T>(&mut self.key) };
		*value_ref = value;
	}
}

pub struct Signal<T> {
	pub(crate) state_key: SignalKey,
	pub(crate) phantom: PhantomData<T>,
}

impl<T: 'static> Signal<T> {
	pub fn new(initial_value: T) -> Self {
		let signal = Context::create_signal(initial_value);

		signal
	}

	pub(crate) unsafe fn get_untracked(&self) -> &T {
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
				state.subscribe(current_scope);
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
}
