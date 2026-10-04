use std::ptr::NonNull;

use crate::reactivity::Source;

#[derive(Clone, Copy)]
pub struct Signal<T> {
	ptr: NonNull<SignalState<T>>,
}

impl<T> Signal<T> {
	pub(crate) fn new(ptr: NonNull<SignalState<T>>) -> Self {
		Self { ptr }
	}
}

pub(crate) struct SignalState<T> {
	source: Source,
	value: T,
}

impl<T> SignalState<T> {
	pub(crate) fn new(value: T) -> Self {
		Self { source: Source::new(), value }
	}
}
