use std::{cell::RefCell, collections::HashSet, ffi::c_void, marker::PhantomData};

use crate::reactivity::{
	allocator::{GeneralStorage, SlabAllocator},
	effect::{Effect, EffectState},
	signal::{Signal, SignalState, SignalValue},
};

thread_local! {
	pub(crate) static CONTEXT: Context = Context::new();
}

// TODO: パフォーマンス計測が必要
pub(crate) struct Context {
	/// for SignalState
	pub(crate) signals: RefCell<SlabAllocator<64>>,

	/// for SignalValue
	pub(crate) values: RefCell<GeneralStorage>,

	/// for EffectState
	pub(crate) effects: RefCell<SlabAllocator<8>>,

	/// dirty subscribers
	pub(crate) pending_remount: RefCell<HashSet<Effect>>,
	pub(crate) pending_logic: RefCell<HashSet<Effect>>,
	pub(crate) pending_render: RefCell<HashSet<Effect>>,

	pub(crate) current_effect: Option<Effect>,
}

impl Context {
	pub(crate) fn new() -> Self {
		Self {
			signals: RefCell::new(SlabAllocator::new()),
			values: RefCell::new(GeneralStorage::new()),
			effects: RefCell::new(SlabAllocator::new()),
			pending_remount: RefCell::new(HashSet::new()),
			pending_logic: RefCell::new(HashSet::new()),
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
}

const _: () = {
	// Slab Allocator Size
	if size_of::<SignalState>() > 64 {
		panic!("SignalState should be =<64");
	}
	if size_of::<EffectState>() > 8 {
		panic!("EffectState should be =<8");
	}
};
