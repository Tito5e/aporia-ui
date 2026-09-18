use std::{cell::RefCell, collections::HashSet, ffi::c_void, marker::PhantomData, ptr::NonNull};

use crate::reactivity::{
	allocator::{GeneralStorage, SlabAllocator},
	effect::{BuildPhase, Effect, EffectState},
	signal::{Signal, SignalState, SignalValue},
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

	/// for EffectState
	pub(crate) effects: RefCell<SlabAllocator<EFFECT_SIZE>>,

	/// dirty subscribers
	pub(crate) pending_build: RefCell<HashSet<Effect>>,
	pub(crate) pending_commit: RefCell<HashSet<Effect>>,
	pub(crate) pending_render: RefCell<HashSet<Effect>>,

	pub(crate) current_effect: Option<Effect>,
}

impl Context {
	pub(crate) fn new() -> Self {
		Self {
			signals: RefCell::new(SlabAllocator::new()),
			values: RefCell::new(GeneralStorage::new()),
			effects: RefCell::new(SlabAllocator::new()),
			pending_build: RefCell::new(HashSet::new()),
			pending_commit: RefCell::new(HashSet::new()),
			pending_render: RefCell::new(HashSet::new()),
			current_effect: None,
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

	pub(crate) fn get_current_effect() -> Option<Effect> {
		CONTEXT.with(|context| context.current_effect)
	}

	pub(crate) fn create_build_effect(effect_ptr: *mut dyn BuildPhase) -> Effect {
		let state = EffectState::Build(unsafe { NonNull::new_unchecked(effect_ptr) });
		let raw_ptr = CONTEXT.with(|context| unsafe { context.effects.borrow_mut().alloc() });
		let state_ptr = raw_ptr as *mut EffectState;
		unsafe { *state_ptr = state };

		Effect::new(state_ptr)
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
