use std::{
	mem::take,
	ptr::{NonNull, addr_of_mut},
};

use crate::reactivity::{
	NONE,
	effect::{EffectHandle, EffectHeader, EffectState},
	pool,
	runtime::Runtime,
	signal::{Signal, SignalHeader, SignalState},
	sink::SinkHeader,
};

struct Owned {
	ptr: *mut u8,
	drop: unsafe fn(*mut Runtime, *mut u8),
}

pub struct Scope {
	rt: NonNull<Runtime>,
	effects: Vec<Owned>,
	values: Vec<Owned>,
}

impl Scope {
	pub fn new(rt: NonNull<Runtime>) -> Self {
		Self { rt, effects: Vec::new(), values: Vec::new() }
	}

	pub fn signal<T: 'static>(&mut self, value: T) -> Signal<T> {
		let rt = self.rt.as_ptr();
		unsafe {
			let ptr = (*pool::<SignalState<T>>(rt))
				.insert(SignalState { header: SignalHeader { subscribers: Vec::new() }, value });
			self.values.push(Owned { ptr: ptr as *mut u8, drop: drop_signal::<T> });

			Signal { ptr: NonNull::new_unchecked(ptr) }
		}
	}

	pub fn alloc<T: 'static>(&mut self, value: T) -> NonNull<T> {
		let rt = self.rt.as_ptr();
		unsafe {
			let ptr = (*pool::<T>(rt)).insert(value);
			self.values.push(Owned { ptr: ptr as *mut u8, drop: drop_plain::<T> });

			NonNull::new_unchecked(ptr)
		}
	}

	pub fn effect_with_sink<F: 'static>(
		&mut self,
		sink: NonNull<SinkHeader>,
		f: F,
	) -> EffectHandle<F> {
		let rt = self.rt.as_ptr();
		unsafe {
			let ptr = (*pool::<EffectState<F>>(rt)).insert(EffectState {
				header: EffectHeader { sink, token: NONE, deps: Vec::new() },
				f,
			});
			self.effects.push(Owned { ptr: ptr as *mut u8, drop: drop_effect::<F> });

			EffectHandle { ptr: NonNull::new_unchecked(ptr) }
		}
	}
}

impl Drop for Scope {
	fn drop(&mut self) {
		let rt = self.rt.as_ptr();
		unsafe {
			while let Some(o) = self.effects.pop() {
				(o.drop)(rt, o.ptr)
			}
			while let Some(o) = self.values.pop() {
				(o.drop)(rt, o.ptr)
			}
		}
	}
}

unsafe fn drop_signal<T: 'static>(rt: *mut Runtime, ptr: *mut u8) {
	let value = unsafe { (*pool::<SignalState<T>>(rt)).remove(ptr as *mut SignalState<T>) };
	debug_assert!(
		value.header.subscribers.is_empty(),
		"signal dropped while still subscribed (scope invariant violated)"
	);
	drop(value);
}

unsafe fn drop_plain<T: 'static>(rt: *mut Runtime, ptr: *mut u8) {
	let value = unsafe { (*pool::<T>(rt)).remove(ptr as *mut T) };
	drop(value);
}

unsafe fn drop_effect<F: 'static>(rt: *mut Runtime, ptr: *mut u8) {
	unsafe {
		let state = ptr as *mut EffectState<F>;
		let header = addr_of_mut!((*state).header);

		let deps = take(&mut (*header).deps);
		for dep in deps {
			let subs = &mut (*dep.as_ptr()).subscribers;
			if let Some(index) = subs.iter().position(|e| e.as_ptr() == header) {
				subs.swap_remove(index);
			}
		}

		if (*header).token != NONE {
			let sink = (*header).sink.as_ptr();
			((*sink).cancel)(sink, (*header).token)
		}

		let value = (*pool::<EffectState<F>>(rt)).remove(state);
		drop(value);
	}
}
