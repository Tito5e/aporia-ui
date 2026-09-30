use std::{
	any::{TypeId, type_name},
	cell::RefCell,
	collections::{HashMap, HashSet},
	marker::PhantomData,
	mem::ManuallyDrop,
	ptr,
};

use log::debug;
use slotmap::SlotMap;

use crate::{
	reactivity::{
		effect::{
			CommitPhase, Effect, EffectData, EffectHandle, EffectKey, EffectPhase, EffectVTable, ReconcilePhase, RenderPhase,
		}, signal::{Signal, SignalKey, SignalState, SignalValue},
	}, storage::{Key, ReserveKey, UnsafeSlotMap, WidgetHandle}, widget::Widget,
};

thread_local! {
	pub(crate) static CONTEXT: Context = Context::new();
}

const SIGNAL_SIZE: usize = 64;

pub(crate) struct Reservation<T: Widget + 'static> {
	key: ReserveKey,
	_phantom: PhantomData<T>,
}

impl<T: Widget + 'static> Reservation<T> {
	fn new(key: ReserveKey) -> Self {
		Self { key, _phantom: PhantomData }
	}

	pub(crate) fn write(self, widget: T) -> WidgetHandle {
		let this = ManuallyDrop::new(self);
		let key = unsafe { ptr::read(&this.key) };
		unsafe { Context::write_widget(key, widget) }
	}

	pub(crate) unsafe fn as_key(&self) -> Key {
		unsafe { self.key.as_key() }
	}
}

impl<T: Widget + 'static> Drop for Reservation<T> {
	fn drop(&mut self) {
		let key = unsafe { ptr::read(&self.key) };
		unsafe { Context::cancel_reservation::<T>(key) }
	}
}

// TODO: パフォーマンス計測が必要
pub(crate) struct Context {
	/// for SignalState
	pub(crate) signals: RefCell<SlotMap<SignalKey, SignalState>>,

	/// for SignalValue
	pub(crate) values: RefCell<HashMap<TypeId, UnsafeSlotMap>>,

	/// for Components
	pub(crate) widgets: RefCell<HashMap<TypeId, UnsafeSlotMap>>,

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
			widgets: RefCell::new(HashMap::new()),

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

	pub(crate) fn create_reconcile_effect<T: Widget + ReconcilePhase + 'static>(key: Key) -> EffectHandle {
		let effect_key = CONTEXT.with(|context| {
			context
				.reconcile_effects
				.borrow_mut()
				.insert(EffectData {
					vtable: EffectVTable::from_method::<T, _>(T::on_reconcile_phase),
					key,
				})
		});

		EffectHandle::new(effect_key, EffectPhase::Reconcile)
	}

	pub(crate) fn create_commit_effect<T: Widget + CommitPhase + 'static>(key: Key) -> EffectHandle {
		let effect_key = CONTEXT.with(|context| {
			context
				.commit_effects
				.borrow_mut()
				.insert(EffectData {
					vtable: EffectVTable::from_method::<T, _>(T::on_commit_phase),
					key,
				})
		});

		EffectHandle::new(effect_key, EffectPhase::Commit)
	}

	pub(crate) fn create_render_effect<T: Widget + RenderPhase + 'static>(key: Key) -> EffectHandle {
		let effect_key = CONTEXT.with(|context| {
			context
				.commit_effects
				.borrow_mut()
				.insert(EffectData {
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

	pub(crate) fn insert_widget<T: Widget + 'static>(widget: T) -> WidgetHandle {
		debug!("Widget insert: {}", type_name::<T>());
		let idx = CONTEXT.with(|context| {
			let mut pools = context.widgets.borrow_mut();
			let pool = pools.entry(TypeId::of::<T>()).or_insert_with(|| UnsafeSlotMap::new::<T>());

			unsafe { pool.insert::<T>(widget) }
		});

		WidgetHandle::new::<T>(idx)
	}

	pub(crate) fn reserve_widget<T: Widget + 'static>() -> Reservation<T> {
		debug!("Widget reserve: {}", type_name::<T>());
		CONTEXT.with(|context| {
			let mut pools = context.widgets.borrow_mut();
			let pool = pools.entry(TypeId::of::<T>()).or_insert_with(|| UnsafeSlotMap::new::<T>());

			let reserve_key = unsafe { pool.reserve::<T>() };

			Reservation::new(reserve_key)
		})
	}

	pub(crate) unsafe fn cancel_reservation<T: Widget + 'static>(reservation: ReserveKey) {
		debug!("Widget reserve cancel: {}", type_name::<T>());
		CONTEXT.with(|context| {
			let mut pools = context.widgets.borrow_mut();
			let pool =
				pools.get_mut(&TypeId::of::<T>()).expect("Widget type has not been registered.");

			unsafe { pool.cancel::<T>(reservation) }
		});
	}

	unsafe fn write_widget<T: Widget + 'static>(
		reservation: ReserveKey,
		widget: T,
	) -> WidgetHandle {
		debug!("Widget write: {}", type_name::<T>());
		let idx = CONTEXT.with(|context| {
			let mut pools = context.widgets.borrow_mut();
			let pool = pools.entry(TypeId::of::<T>()).or_insert_with(|| UnsafeSlotMap::new::<T>());

			unsafe { pool.write(reservation, widget) }
		});

		WidgetHandle::new::<T>(idx)
	}

	pub(crate) unsafe fn remove_widget<T: Widget + 'static>(key: Key) -> T {
		debug!("Widget remove: {}", type_name::<T>());
		CONTEXT.with(|context| {
			let mut pools = context.widgets.borrow_mut();
			let pool =
				pools.get_mut(&TypeId::of::<T>()).expect("Widget type has not been registered.");

			unsafe { pool.remove(key) }
		})
	}

	/// #Safety
	/// `T`は`handle`の確保時に使われた型と一致していなければならない
	/// この関数で確保した参照は"正しく"使われる必要がある
	/// * handleが`remove_widget`される前に参照を破棄する必要がある
	/// * `Context::insert_widget`、`Context::reserve_widget`のどちらかの関数を呼び出す前に参照を破棄する必要がある
	pub(crate) unsafe fn get_widget<T: Widget + 'static>(key: &Key) -> &T {
		debug!("Widget referenced: {}", type_name::<T>());
		let ptr = CONTEXT.with(|context| {
			let mut pools = context.widgets.borrow_mut();
			let pool =
				pools.get_mut(&TypeId::of::<T>()).expect("Widget type has not been registered.");

			unsafe { pool.get(*key) as *const T }
		});
		unsafe { &*ptr }
	}

	pub(crate) unsafe fn get_widget_mut<T: Widget + 'static>(key: &mut Key) -> &mut T {
		debug!("Widget mutable referenced: {}", type_name::<T>());
		let ptr = CONTEXT.with(|context| {
			let mut pools = context.widgets.borrow_mut();
			let pool =
				pools.get_mut(&TypeId::of::<T>()).expect("Widget type has not been registered.");

			unsafe { pool.get(*key) as *const T as *mut T }
		});

		unsafe { &mut *ptr }
	}
}

const _: () = {
	// Slab Allocator Size
	if size_of::<SignalState>() > SIGNAL_SIZE {
		panic!("SignalState size over");
	}
};
