use std::ptr::NonNull;

use crate::reactivity::{context::Context, signal::Signal};

pub(crate) enum EffectState {
	Build(NonNull<dyn BuildPhase>),
	Commit(NonNull<dyn CommitPhase>),
	Render(NonNull<dyn RenderPhase>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BuildEffect(NonNull<dyn BuildPhase>);

impl BuildEffect {
	pub fn new_validated(ptr: *mut dyn BuildPhase) -> Self {
		Self(unsafe { NonNull::new_unchecked(ptr) })
	}

	pub fn new(ptr: *mut dyn BuildPhase) -> Self {
		let effect = BuildEffect::new_validated(ptr);
		Context::invalidate_build_effect(effect);

		effect
	}

	pub fn subscribe<T>(&self, signal: Signal<T>) {
		signal.subscribe(Effect::Build(*self));
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CommitEffect(NonNull<dyn CommitPhase>);

impl CommitEffect {
	pub fn new(ptr: *mut dyn CommitPhase) -> Self {
		Self(unsafe { NonNull::new_unchecked(ptr) })
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct RenderEffect(NonNull<dyn RenderPhase>);

impl RenderEffect {
	pub fn new(ptr: *mut dyn RenderPhase) -> Self {
		Self(unsafe { NonNull::new_unchecked(ptr) })
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Effect {
	Build(BuildEffect),
	Commit(CommitEffect),
	Render(RenderEffect),
}

impl Effect {
	pub fn invalidate(&self) {
		match self {
			Effect::Build(build_effect) => Context::invalidate_build_effect(*build_effect),
			Effect::Commit(commit_effect) => Context::invalidate_commit_effect(*commit_effect),
			Effect::Render(render_effect) => Context::invalidate_render_effect(*render_effect),
		}
	}
}

pub trait BuildPhase {
	fn on_build_phase(&mut self);
}

pub trait CommitPhase {
	fn on_commit_phase(&mut self);
}

pub trait RenderPhase {
	fn on_render_phase(&mut self);
}
