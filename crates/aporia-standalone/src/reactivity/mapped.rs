use std::ptr::null_mut;

use crate::reactivity::{Source, effect::EffectHeader};

#[derive(Clone, Copy)]
pub struct Mapped<F> {
	f: F,
}

impl<R, F: Fn(&MappedCx) -> R + Clone> Mapped<F> {
	pub fn new(f: F) -> Self {
		Self { f }
	}
}

impl<R, F: Fn(&MappedCx) -> R> Source for Mapped<F> {
	type Out = R;

	fn read(&self) -> R {
		(self.f)(&MappedCx::untracked())
	}

	fn subscribe(&self, effect: &mut EffectHeader) -> R {
		(self.f)(&MappedCx::new(effect))
	}
}

pub struct MappedCx {
	effect: *mut EffectHeader,
}

impl MappedCx {
	pub const fn new(effect: *mut EffectHeader) -> Self {
		Self { effect }
	}

	fn untracked() -> Self {
		Self { effect: null_mut() }
	}

	pub fn read<S: Source>(&self, source: S) -> S::Out {
		if self.effect.is_null() {
			source.read()
		} else {
			unsafe { source.subscribe(&mut *self.effect) }
		}
	}
}
