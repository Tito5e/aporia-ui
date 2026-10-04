use std::ptr::NonNull;

use crate::reactivity::Effect;

pub struct Source {
	subscribers: Vec<NonNull<dyn Effect>>,
}

impl Source {
	#[inline(always)]
	#[must_use]
	pub fn new() -> Self {
		Self { subscribers: Vec::new() }
	}

	#[inline]
	pub fn notify(&self) {
		for effect in &self.subscribers {
			unsafe {
				let effect = effect.as_ref();
				effect.invalidate();
			}
		}
	}

	#[inline]
	pub fn subscribe<E: Effect + 'static>(&mut self, effect: &mut E) {
		if effect.connect(self) {
			self.subscribers.push(NonNull::from(effect));
		}
	}
}
