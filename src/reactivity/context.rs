use std::{cell::RefCell, collections::HashSet, ffi::c_void, marker::PhantomData};

use crate::{
	component::WidgetHandle,
	reactivity::{
		allocator::{GeneralStorage, SlabAllocator},
		effect::{BuildEffect, BuildPhase, CommitEffect, EffectState, RenderEffect},
		signal::{Signal, SignalState, SignalValue},
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
	pub(crate) signals: RefCell<SlabAllocator<SIGNAL_SIZE>>,

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
			signals: RefCell::new(SlabAllocator::new()),
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
		let raw_ptr = CONTEXT.with(|context| unsafe { context.signals.borrow_mut().alloc() });
		let state_ptr = raw_ptr as *mut SignalState;
		unsafe { *state_ptr = state };

		Signal { state_ptr, phantom: PhantomData }
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
		let ptr = CONTEXT.with(|context| {
			context.widgets.borrow_mut().alloc_with(widget, |p| p as *mut dyn Widget)
		});
		WidgetHandle(ptr)
	}

	pub(crate) fn free_widget(handle: &WidgetHandle) {
		CONTEXT.with(|context| unsafe { context.widgets.borrow_mut().free(handle.0) });
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
