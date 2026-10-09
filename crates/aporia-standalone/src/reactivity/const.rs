use crate::reactivity::{Source, effect::EffectHeader};

#[derive(Clone, Copy)]
pub struct Const<T>(T);

impl<T> Const<T> {
	pub fn new(value: T) -> Self {
		Self(value)
	}
}

impl<T: Clone> Source for Const<T> {
	type Out = T;

	fn read(&self) -> T {
		self.0.clone()
	}

	fn subscribe(&self, _: &mut EffectHeader) -> T {
		self.0.clone()
	}
}
