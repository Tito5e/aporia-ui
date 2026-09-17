use std::ptr::NonNull;

use crate::widget::Remountable;

pub(crate) enum ScopeState {
	Remount { is_dirty: bool, data: NonNull<dyn Remountable> },
	Logic { is_dirty: bool },
	Render { is_dirty: bool },
}

const _: () = {
	if size_of::<ScopeState>() > 32 {
		// link: Slab Allocator Size
		panic!("ScopeData should be =<32");
	}
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Scope(*mut ScopeState);
