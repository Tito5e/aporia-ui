use std::{cell::RefCell, collections::HashSet, ffi::c_void, marker::PhantomData, ptr};

use log::debug;
use slotmap::SlotMap;

use crate::{
	component::WidgetHandle,
	reactivity::{
		allocator::GeneralStorage,
		effect::{BuildEffect, BuildPhase, CommitEffect, EffectState, RenderEffect},
		signal::{Signal, SignalKey, SignalState, SignalValue},
	},
	widget::Widget,
};

thread_local! {
	pub(crate) static CONTEXT: Context = Context::new();
}

const SIGNAL_SIZE: usize = 64;
const EFFECT_SIZE: usize = 32;

// TODO: パフォーマンス計測が必要
pub(crate) struct Context {
	/// for SignalState
	pub(crate) signals: RefCell<SlotMap<SignalKey, SignalState>>,

	/// for SignalValue
	pub(crate) values: RefCell<GeneralStorage>,

	/// for Components
	pub(crate) widgets: RefCell<GeneralStorage>,

	/// dirty subscribers
	pub(crate) pending_build: RefCell<HashSet<BuildEffect>>,
	pub(crate) pending_commit: RefCell<HashSet<CommitEffect>>,
	pub(crate) pending_render: RefCell<HashSet<RenderEffect>>,

	pub(crate) current_effect: RefCell<Option<BuildEffect>>,
}

impl Context {
	pub(crate) fn new() -> Self {
		Self {
			signals: RefCell::new(SlotMap::with_key()),
			values: RefCell::new(GeneralStorage::new()),
			widgets: RefCell::new(GeneralStorage::new()),
			pending_build: RefCell::new(HashSet::new()),
			pending_commit: RefCell::new(HashSet::new()),
			pending_render: RefCell::new(HashSet::new()),
			current_effect: RefCell::new(None),
		}
	}

	pub(crate) fn create_signal<T>(initial_value: T) -> Signal<T> {
		let value_ptr = CONTEXT.with(|context| context.values.borrow_mut().alloc(initial_value));
		let value = SignalValue::new(value_ptr as *mut c_void);
		let state = SignalState { value, subscribers: HashSet::new() };
		let state_key = CONTEXT.with(|context| context.signals.borrow_mut().insert(state));

		Signal { state_key, phantom: PhantomData }
	}

	pub(crate) fn get_current_effect() -> Option<BuildEffect> {
		CONTEXT.with(|context| *context.current_effect.borrow())
	}

	pub(crate) fn set_current_effect(effect: Option<BuildEffect>) {
		CONTEXT.with(|context| context.current_effect.replace(effect));
	}

	pub(crate) fn create_build_effect(effect_ptr: *mut dyn BuildPhase) -> BuildEffect {
		BuildEffect::new(effect_ptr)
	}

	pub(crate) fn invalidate_build_effect(effect: BuildEffect) {
		CONTEXT.with(|context| context.pending_build.borrow_mut().insert(effect));
	}

	pub(crate) fn invalidate_commit_effect(effect: CommitEffect) {
		CONTEXT.with(|context| context.pending_commit.borrow_mut().insert(effect));
	}

	pub(crate) fn invalidate_render_effect(effect: RenderEffect) {
		CONTEXT.with(|context| context.pending_render.borrow_mut().insert(effect));
	}

	pub(crate) fn allocate_widget<T: Widget + 'static>(widget: T) -> WidgetHandle {
		debug!("Alloc: {}, {}", size_of::<T>(), align_of::<T>());
		let ptr = CONTEXT.with(|context| {
			context.widgets.borrow_mut().alloc_with(widget, |p| p as *mut dyn Widget)
		});
		WidgetHandle(ptr)
	}

	pub(crate) fn allocate_widget_uninit<T: Widget + 'static>() -> WidgetHandle {
		debug!("Alloc Uninit: {}, {}", size_of::<T>(), align_of::<T>());
		let ptr = CONTEXT.with(|context| {
			context.widgets.borrow_mut().alloc_uninit::<_, T, _>(|p| p as *mut dyn Widget)
		});
		WidgetHandle(ptr)
	}

	pub(crate) fn free_widget(handle: &WidgetHandle) {
		let size = size_of_val(unsafe { &*handle.0 });
		let align = align_of_val(unsafe { &*handle.0 });
		unsafe { ptr::drop_in_place(handle.0) };
		debug!("Free: {}, {}", size, align);
		CONTEXT.with(|context| unsafe {
			context.widgets.borrow_mut().free_raw(handle.0 as *mut u8, size, align);
		});
	}
}

const _: () = {
	// Slab Allocator Size
	if size_of::<SignalState>() > SIGNAL_SIZE {
		panic!("SignalState size over");
	}
	if size_of::<EffectState>() > EFFECT_SIZE {
		panic!("EffectState size over");
	}
};
