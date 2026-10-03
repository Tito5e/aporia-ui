use std::ptr::null_mut;

use crate::reactivity::{
	effect::{EffectHeader, EffectState},
	sink::SinkHeader,
};

pub struct ReconcileEffect {}

impl ReconcileEffect {
	fn run(&mut self) {
		unsafe { todo!("Reconcile isnt implemented") }
	}
}

#[repr(C)]
pub struct ReconcileQueue {
	sink: SinkHeader,
	items: Vec<*mut EffectState<ReconcileEffect>>,
}

impl ReconcileQueue {
	pub(crate) const fn new() -> Self {
		Self { sink: SinkHeader { mark: Self::mark, cancel: Self::cancel }, items: Vec::new() }
	}

	pub(crate) const fn header(&self) -> SinkHeader {
		self.sink
	}

	unsafe fn mark(sink: *mut SinkHeader, effect: *mut EffectHeader) -> u32 {
		unsafe {
			let queue = sink as *mut Self;
			(*queue).items.push(effect as *mut EffectState<ReconcileEffect>);
			((*queue).items.len() - 1) as u32
		}
	}

	unsafe fn cancel(sink: *mut SinkHeader, token: u32) {
		let queue = sink as *mut Self;
		unsafe { (&mut (*queue).items)[token as usize] = null_mut() }
	}

	unsafe fn flush(queue: *mut Self) -> bool {
		unsafe {
			let mut index = 0;
			while index < (*queue).items.len() {
				let state = (&(*queue).items)[index];
				index += 1;
				if state.is_null() {
					continue;
				}
				(*state).header.begin_run();
				(*state).f.run();
			}
			let ran = index > 0;
			(*queue).items.clear();
			ran
		}
	}
}
