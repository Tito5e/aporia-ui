use crate::{
	application::StandaloneCx,
	component::{Builder, View},
	reactivity::{Signal, Source, effect::EffectHandle, scope::Scope},
	reconcile::ReconcileEffect,
};

pub struct ViewCx<'a, State> {
	pub(crate) ctx: &'a mut StandaloneCx,
	pub(crate) scope: &'a mut Scope,
	pub(crate) reconciler: &'a mut EffectHandle<ReconcileEffect>,
	pub(crate) global_state: &'a mut State,
}

impl<'a, State> ViewCx<'a, State> {
	#[inline(always)]
	#[must_use]
	pub fn render(&mut self, builder: impl Builder) -> View {
		View(builder.build(self.ctx))
	}

	#[inline(always)]
	#[must_use]
	pub fn signal<T: 'static>(&mut self, value: T) -> Signal<T> {
		self.scope.signal(value)
	}

	#[inline(always)]
	#[must_use]
	pub fn read<S: Source>(&mut self, source: S) -> S::Out {
		source.subscribe(self.reconciler)
	}

	#[inline(always)]
	#[must_use]
	pub fn global_state(&self) -> &State {
		self.global_state
	}
}
