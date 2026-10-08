mod context;
pub use context::ReconcileCx;

use std::ptr::{NonNull, null_mut};

use crate::reactivity::{
	effect::{EffectHeader, EffectState},
	sink::SinkHeader,
};

pub trait Reconcile<S> {
	fn reconcile<'a, 'b>(&'a mut self, cx: &'a mut ReconcileCx<'b, S>);
}

pub struct ReconcileEffect<S> {
	ptr: NonNull<dyn Reconcile<S>>,
}

impl<S> ReconcileEffect<S> {
	#[inline(always)]
	pub(crate) const fn new(ptr: NonNull<dyn Reconcile<S>>) -> Self {
		Self { ptr }
	}

	#[inline(always)]
	fn run(&mut self, cx: &mut ReconcileCx<S>) {
		unsafe { (*self.ptr.as_ptr()).reconcile(cx) };
	}
}

#[repr(C)]
pub struct ReconcileQueue<S> {
	sink: SinkHeader,
	items: Vec<*mut EffectState<ReconcileEffect<S>>>,
}

impl<S> ReconcileQueue<S> {
	pub(crate) const fn new() -> Self {
		Self { sink: SinkHeader { mark: Self::mark, cancel: Self::cancel }, items: Vec::new() }
	}

	pub(crate) const fn header(&self) -> SinkHeader {
		self.sink
	}

	unsafe fn mark(sink: *mut SinkHeader, effect: *mut EffectHeader) -> u32 {
		unsafe {
			let queue = sink as *mut Self;
			(*queue).items.push(effect as *mut EffectState<ReconcileEffect<S>>);
			((*queue).items.len() - 1) as u32
		}
	}

	unsafe fn cancel(sink: *mut SinkHeader, token: u32) {
		let queue = sink as *mut Self;
		unsafe { (&mut (*queue).items)[token as usize] = null_mut() }
	}

	unsafe fn flush<'a>(queue: *mut Self, cx: &mut ReconcileCx<S>) -> bool {
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
