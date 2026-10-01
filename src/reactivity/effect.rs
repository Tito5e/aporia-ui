use std::{collections::HashSet, ffi::c_void, mem::transmute, ptr::NonNull};

use log::debug;
use slotmap::new_key_type;

use crate::{
	reactivity::{
		context::{CONTEXT, Context},
		signal::{Signal, SignalKey, SignalState},
	},
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

impl Drop for EffectHandle {
	fn drop(&mut self) {
		for dep in self.deps.iter() {
			let state: &mut SignalState = CONTEXT.with(|context| unsafe {
				transmute(context.signals.borrow_mut().get_unchecked_mut(*dep) as &mut SignalState)
			});

			state.subscribers.remove(&self.as_ptr());
		}
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
	run: fn(ptr: NonNull<c_void>),
}

impl EffectVTable {
	pub(crate) const fn from_method<T, F>(_: F) -> &'static Self
	where
		T: Widget + 'static,
		F: Fn(&mut T) + Copy + 'static,
	{
		const { assert!(core::mem::size_of::<F>() == 0) }

		&Self {
			run: |ptr| {
				let widget = unsafe { &mut *ptr.as_ptr().cast::<T>() };
				(widget);
			},
		}
	}
}

pub struct EffectData {
	pub(crate) vtable: fn(ptr: NonNull<c_void>),
	pub(crate) ptr: NonNull<c_void>,
}

impl EffectData {
	pub(crate) const fn from_reconcile<T: Widget + ReconcilePhase + 'static>(
		ptr: NonNull<c_void>,
	) -> Self {
		Self {
			ptr,
			vtable: |ptr| {
				let widget = unsafe { &mut *ptr.as_ptr().cast::<T>() };
				widget.on_reconcile_phase();
			},
		}
	}
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
