mod context;
pub use context::ReconcileCx;

use std::ptr::{NonNull, null_mut};

use crate::{
	context::StandaloneCx,
	reactivity::{
		effect::{EffectHeader, EffectState},
		sink::SinkHeader,
	},
};

pub trait Reconcile {
	fn reconcile(&mut self, cx: &mut ReconcileCx);
}

pub struct ReconcileEffect {
	ptr: NonNull<dyn Reconcile>,
}

impl ReconcileEffect {
	#[inline(always)]
	pub(crate) const fn new(ptr: NonNull<dyn Reconcile>) -> Self {
		Self { ptr }
	}

	#[inline(always)]
	fn run(&mut self, cx: &mut StandaloneCx) {
		let mut cx = ReconcileCx { cx };
		unsafe { (*self.ptr.as_ptr()).reconcile(&mut cx) };
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

	unsafe fn flush(queue: *mut Self, cx: &mut StandaloneCx) -> bool {
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
