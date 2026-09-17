use std::ptr::NonNull;

use crate::widget::Remountable;

pub(crate) enum SubscriberState {
	Remount { is_dirty: bool, data: NonNull<dyn Remountable> },
	Logic { is_dirty: bool },
	Render { is_dirty: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Subscriber(*mut SubscriberState);
