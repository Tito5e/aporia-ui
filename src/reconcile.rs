use std::ptr::{NonNull, null_mut};

use crate::{
	reactivity::{
		effect::{EffectHeader, EffectState},
		sink::SinkHeader,
	},
	standalone::context::Context,
};

pub trait Reconcile {
	fn reconcile(&mut self, cx: &mut Context);
}

pub struct ReconcileEffect {
	ptr: NonNull<dyn Reconcile>,
}

impl ReconcileEffect {
	pub(crate) fn new(ptr: NonNull<dyn Reconcile>) -> Self {
		Self { ptr }
	}

	fn run(&mut self, cx: &mut Context) {
		unsafe {
			let state = self.ptr.as_mut();
			state.reconcile(cx);
		}
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

	unsafe fn flush(queue: *mut Self, cx: &mut Context) -> bool {
		unsafe {
			let mut index = 0;
			while index < (*queue).items.len() {
				let state = (&(*queue).items)[index];
				index += 1;
				if state.is_null() {
					continue;
				}
				(*state).header.begin_run();
				(*state).f.run(cx);
			}
			let ran = index > 0;
			(*queue).items.clear();
			ran
		}
	}
}
