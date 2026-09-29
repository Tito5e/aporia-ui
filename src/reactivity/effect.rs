use slotmap::new_key_type;

use crate::{
	reactivity::{context::Context, signal::Signal},
	storage::Key,
	widget::Widget,
};

new_key_type! {
	pub(crate) struct EffectKey;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReconcileHandle {
	pub(crate) key: EffectKey,
	vtable: &'static ReconcileVTable,
}

impl ReconcileHandle {
	pub fn new<T: BuildPhase + 'static>(key: EffectKey) -> Self {
		Self { key, vtable: ReconcileVTable::build::<T>() }
	}

	pub fn invalidate(&self) {
		Context::invalidate_build_effect(*self);
	}

	pub fn subscribe<T>(&self, signal: Signal<T>) {
		signal.subscribe(Effect::Build(*self));
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ReconcileVTable {
	on_reconcile: fn(key: Key),
}

impl ReconcileVTable {
	pub(crate) const fn from_method<T: Widget + BuildPhase + 'static>() -> &'static Self {
		&Self {
			on_reconcile: |mut key| {
				let widget = unsafe { Context::get_widget_mut::<T>(&mut key) };
				widget.on_build_phase();
			},
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CommitHandle {
	key: EffectKey,
	vtable: &'static CommitVTable,
}

impl CommitHandle {
	pub fn new<T: CommitPhase + 'static>(key: EffectKey) -> Self {
		Self { key, vtable: CommitVTable::build::<T>() }
	}

	pub fn invalidate(&self) {
		Context::invalidate_commit_effect(*self);
	}

	pub fn subscribe<T>(&self, signal: Signal<T>) {
		signal.subscribe(Effect::Commit(*self));
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CommitVTable {
	on_commit_phase: fn(key: Key),
}

impl CommitVTable {
	const fn build<T: CommitPhase + 'static>() -> &'static Self {
		&Self {
			on_commit_phase: |mut key| {
				let widget = unsafe { Context::get_widget_mut::<T>(&mut key) };
				widget.on_commit_phase();
			},
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct RenderHandle {
	key: EffectKey,
	vtable: &'static RenderVTable,
}

impl RenderHandle {
	pub fn new<T: RenderPhase + 'static>(key: EffectKey) -> Self {
		Self { key, vtable: RenderVTable::build::<T>() }
	}

	pub fn invalidate(&self) {
		Context::invalidate_render_effect(*self);
	}

	pub fn subscribe<T>(&self, signal: Signal<T>) {
		signal.subscribe(Effect::Render(*self));
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct RenderVTable {
	on_render_phase: fn(key: Key),
}

impl RenderVTable {
	const fn build<T: RenderPhase + 'static>() -> &'static Self {
		&Self {
			on_render_phase: |mut key| {
				let widget = unsafe { Context::get_widget_mut::<T>(&mut key) };
				widget.on_render_phase();
			},
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Effect {
	Build(ReconcileHandle),
	Commit(CommitHandle),
	Render(RenderHandle),
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
