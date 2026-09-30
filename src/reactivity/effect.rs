use std::{collections::HashSet, mem::transmute};

use slotmap::new_key_type;

use crate::{
	reactivity::{
		context::{CONTEXT, Context},
		signal::{Signal, SignalKey, SignalState},
	},
	storage::Key,
	widget::Widget,
};

new_key_type! {
	pub(crate) struct EffectKey;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectPhase {
	Reconcile,
	Commit,
	Render,
}

#[derive(Debug, PartialEq, Eq)]
pub struct EffectHandle {
	pub(crate) key: EffectKey,
	pub(crate) phase: EffectPhase,
	pub(crate) deps: HashSet<SignalKey>,
}

impl EffectHandle {
	pub fn new(key: EffectKey, phase: EffectPhase) -> Self {
		Self { key, phase, deps: HashSet::new() }
	}

	pub fn invalidate(&self) {
		match self.phase {
			EffectPhase::Reconcile => Context::invalidate_build_effect(self.key),
			EffectPhase::Commit => Context::invalidate_commit_effect(self.key),
			EffectPhase::Render => Context::invalidate_render_effect(self.key),
		}
	}

	pub fn subscribe<T: 'static>(&self, signal: Signal<T>) {
		unsafe {
			let state: &mut SignalState = CONTEXT.with(|context| {
				transmute(context.signals.borrow_mut().get_unchecked_mut(signal.state_key)
					as &mut SignalState)
			});
			state.subscribe(self.as_ptr());
		}
	}

	pub const fn as_ptr(&self) -> Effect {
		Effect { key: self.key, phase: self.phase }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Effect {
	pub(crate) key: EffectKey,
	pub(crate) phase: EffectPhase,
}

impl Effect {
	pub fn invalidate(&self) {
		match self.phase {
			EffectPhase::Reconcile => Context::invalidate_build_effect(self.key),
			EffectPhase::Commit => Context::invalidate_commit_effect(self.key),
			EffectPhase::Render => Context::invalidate_render_effect(self.key),
		}
	}

	pub fn subscribe<T: 'static>(&self, signal: Signal<T>) {
		unsafe {
			let state: &mut SignalState = CONTEXT.with(|context| {
				transmute(context.signals.borrow_mut().get_unchecked_mut(signal.state_key)
					as &mut SignalState)
			});
			state.subscribe(*self);
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct EffectVTable {
	run: fn(key: Key),
}

impl EffectVTable {
	pub(crate) const fn from_method<T, F>(_: F) -> &'static Self
	where
		T: Widget + 'static,
		F: Fn(&mut T) + Copy + 'static,
	{
		const { assert!(core::mem::size_of::<F>() == 0) }

		&Self {
			run: |mut key| {
				let widget = unsafe { Context::get_widget_mut::<T>(&mut key) };
				let f: F = unsafe { core::mem::zeroed() };
				f(widget);
			},
		}
	}
}

pub struct EffectData {
	pub(crate) vtable: &'static EffectVTable,
	pub(crate) key: Key,
}

pub trait ReconcilePhase {
	fn on_reconcile_phase(&mut self);
}

pub trait CommitPhase {
	fn on_commit_phase(&mut self);
}

pub trait RenderPhase {
	fn on_render_phase(&mut self);
}
