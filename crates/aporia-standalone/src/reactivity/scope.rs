use std::ptr::NonNull;

use crate::reactivity::{Context, Effect, Signal, SignalState};

struct Owned {
	ptr: *mut u8,
	drop: unsafe fn(*mut Context, *mut u8),
}

pub struct Scope {
	cx: NonNull<Context>,
	effects: Vec<Owned>,
	values: Vec<Owned>,
}

impl Scope {
	pub fn new(cx: NonNull<Context>) -> Self {
		Self { cx, effects: Vec::new(), values: Vec::new() }
	}

	pub fn signal<T: 'static>(&mut self, value: T) -> Signal<T> {
		let cx = unsafe { self.cx.as_mut() };
		let ptr = unsafe { cx.pool::<SignalState<T>>().insert(SignalState::new(value)) };
		self.values.push(Owned { ptr: ptr as *mut u8, drop: drop_signal::<T> });

		Signal::new(unsafe { NonNull::new_unchecked(ptr) })
	}

	pub fn effect_with_sink<E: Effect + 'static>(
		&mut self,
		sink: NonNull<dyn Sink>,
		effect: E,
	) -> EffectHandle<E> {
		unsafe {
			let cx = unsafe { self.cx.as_mut() };
			let ptr = unsafe { cx.pool::<E>().insert(effect) };
			self.effects.push(Owned { ptr: ptr as *mut u8, drop: drop_effect::<E> });

			EffectHandle { ptr: NonNull::new_unchecked(ptr) }
		}
	}
}

unsafe fn drop_signal<T: 'static>(cx: *mut Context, ptr: *mut u8) {
	let cx = unsafe { &mut *cx };
	let value = unsafe { cx.pool::<SignalState<T>>().remove(ptr as *mut SignalState<T>) };
	drop(value);
}

unsafe fn drop_effect<E: Effect + 'static>(cx: *mut Context, ptr: *mut u8) {
	unsafe {
		let deps = take
	}
}
