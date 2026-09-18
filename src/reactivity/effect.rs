use std::ptr::NonNull;

pub(crate) enum EffectState {
	Build(NonNull<dyn BuildPhase>),
	Commit(NonNull<dyn CommitPhase>),
	Render(NonNull<dyn RenderPhase>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Effect(*mut EffectState);

impl Effect {
	pub fn new(ptr: *mut EffectState) -> Self {
		Self(ptr)
	}
}

pub trait BuildPhase {
	fn on_build(&mut self);
}

pub trait CommitPhase {
	fn on_commit(&mut self);
}

pub trait RenderPhase {
	fn on_render(&mut self);
}
