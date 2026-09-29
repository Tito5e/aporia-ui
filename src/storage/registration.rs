use std::marker::PhantomData;

use crate::{
	reactivity::effect::{BuildPhase, ReconcileHandle},
	storage::{Key, WidgetHandle},
	widget::Widget,
};

pub struct Registration<T> {
	key: Key,
	_marker: PhantomData<T>,
}

impl<T: Widget + 'static> Registration<T> {
	pub fn handles(self) -> WidgetHandle {
		WidgetHandle::new::<T>(self.key)
	}
}

impl<T: Widget + BuildPhase + 'static> Registration<T> {
	pub fn handles(self) -> (WidgetHandle, ReconcileHandle) {
		(WidgetHandle::new::<T>(self.key), ReconcileHandle::new::<T>(self.key))
	}
}
