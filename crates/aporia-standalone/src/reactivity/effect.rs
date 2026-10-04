use crate::reactivity::Source;

pub trait Effect {
	fn connect(&mut self, source: &Source) -> bool
	where
		Self: Sized;
	fn invalidate(&self);
}
