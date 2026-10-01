use std::{
	any::{TypeId, type_name},
	cell::RefCell,
	collections::{HashMap, HashSet},
	ffi::c_void,
	marker::PhantomData,
	ptr::NonNull,
};

use log::debug;
use slotmap::SlotMap;

use crate::{
	reactivity::{
		effect::{
			CommitPhase, Effect, EffectData, EffectHandle, EffectKey, EffectPhase, EffectVTable,
			ReconcilePhase, RenderPhase,
		},
		signal::{Signal, SignalKey, SignalState, SignalValue},
	},
	storage::{Key, UnsafeSlotMap},
	widget::Widget,
};

thread_local! {
	pub(crate) static CONTEXT: Context = Context::new();
}

// TODO: パフォーマンス計測が必要
pub(crate) struct Context {
	/// for SignalState
	pub(crate) signals: RefCell<SlotMap<SignalKey, SignalState>>,

	/// for SignalValue
	pub(crate) values: RefCell<HashMap<TypeId, UnsafeSlotMap>>,

	/// for Effects
	pub(crate) reconcile_effects: RefCell<SlotMap<EffectKey, EffectData>>,
	pub(crate) commit_effects: RefCell<SlotMap<EffectKey, EffectData>>,
	pub(crate) render_effects: RefCell<SlotMap<EffectKey, EffectData>>,

	/// dirty subscribers
	pub(crate) pending_build: RefCell<HashSet<EffectKey>>,
	pub(crate) pending_commit: RefCell<HashSet<EffectKey>>,
	pub(crate) pending_render: RefCell<HashSet<EffectKey>>,

	pub(crate) current_effect: RefCell<Option<Effect>>,
}

impl Context {
	pub(crate) fn new() -> Self {
		Self {
			signals: RefCell::new(SlotMap::with_key()),
			values: RefCell::new(HashMap::new()),

			reconcile_effects: RefCell::new(SlotMap::with_key()),
			commit_effects: RefCell::new(SlotMap::with_key()),
			render_effects: RefCell::new(SlotMap::with_key()),

			pending_build: RefCell::new(HashSet::new()),
			pending_commit: RefCell::new(HashSet::new()),
			pending_render: RefCell::new(HashSet::new()),
			current_effect: RefCell::new(None),
		}
	}

	pub(crate) fn create_signal<T: 'static>(initial_value: T) -> Signal<T> {
		let value_key = CONTEXT.with(|context| {
			let mut value_pools = context.values.borrow_mut();
			let value_pool =
				value_pools.entry(TypeId::of::<T>()).or_insert_with(|| UnsafeSlotMap::new::<T>());

			unsafe { value_pool.insert::<T>(initial_value) }
		});
		let value = SignalValue::new(value_key);
		let state = SignalState { value, subscribers: HashSet::new() };
		let state_key = CONTEXT.with(|context| context.signals.borrow_mut().insert(state));

		Signal { state_key, phantom: PhantomData }
	}

	pub(crate) unsafe fn read_signal<T: 'static>(key: &mut Key) -> &mut T {
		debug!("Signal read: {}", type_name::<T>());
		let ptr = CONTEXT.with(|context| {
			let mut pools = context.values.borrow_mut();
			let pool =
				pools.get_mut(&TypeId::of::<T>()).expect("Widget type has not been registered.");

			unsafe { pool.get(*key) as *const T as *mut T }
		});
		unsafe { &mut *ptr }
	}

	pub(crate) fn get_current_effect() -> Option<Effect> {
		CONTEXT.with(|context| *context.current_effect.borrow())
	}

	pub(crate) fn set_current_effect(effect: Option<Effect>) {
		CONTEXT.with(|context| context.current_effect.replace(effect));
	}

	pub(crate) fn create_reconcile_effect<T: Widget + ReconcilePhase + 'static>(
		key: NonNull<c_void>,
	) -> EffectHandle {
		let effect_key = CONTEXT.with(|context| {
			context.reconcile_effects.borrow_mut().insert(EffectData {
				vtable: EffectVTable::from_method::<T, _>(T::on_reconcile_phase),
				key,
			})
		});

		EffectHandle::new(effect_key, EffectPhase::Reconcile)
	}

	pub(crate) fn create_commit_effect<T: Widget + CommitPhase + 'static>(
		key: Key,
	) -> EffectHandle {
		let effect_key = CONTEXT.with(|context| {
			context.commit_effects.borrow_mut().insert(EffectData {
				vtable: EffectVTable::from_method::<T, _>(T::on_commit_phase),
				key,
			})
		});

		EffectHandle::new(effect_key, EffectPhase::Commit)
	}

	pub(crate) fn create_render_effect<T: Widget + RenderPhase + 'static>(
		key: Key,
	) -> EffectHandle {
		let effect_key = CONTEXT.with(|context| {
			context.commit_effects.borrow_mut().insert(EffectData {
				vtable: EffectVTable::from_method::<T, _>(T::on_render_phase),
				key,
			})
		});

		EffectHandle::new(effect_key, EffectPhase::Render)
	}

	pub(crate) fn invalidate_build_effect(key: EffectKey) {
		CONTEXT.with(|context| context.pending_build.borrow_mut().insert(key));
	}

	pub(crate) fn invalidate_commit_effect(key: EffectKey) {
		CONTEXT.with(|context| context.pending_commit.borrow_mut().insert(key));
	}

	pub(crate) fn invalidate_render_effect(key: EffectKey) {
		CONTEXT.with(|context| context.pending_render.borrow_mut().insert(key));
	}
}
