use std::{cell::RefCell, collections::HashSet, ffi::c_void, marker::PhantomData};

use crate::reactivity::{
	allocator::{GeneralStorage, SlabAllocator},
	signal::{Signal, SignalState, SignalValue},
	subscriber::{Subscriber, SubscriberState},
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

	/// for SubscriberState
	pub(crate) subscribers: RefCell<SlabAllocator<32>>,

	/// dirty scopes
	pub(crate) pending_subscribers: RefCell<HashSet<Subscriber>>,

	pub(crate) current_mounter: Option<Subscriber>,
}

impl Context {
	pub(crate) fn new() -> Self {
		Self {
			signals: RefCell::new(SlabAllocator::new()),
			values: RefCell::new(GeneralStorage::new()),
			subscribers: RefCell::new(SlabAllocator::new()),
			pending_subscribers: RefCell::new(HashSet::new()),
			current_mounter: None,
		}
	}

	pub(crate) fn create_signal<T>(initial_value: T) -> Signal<T> {
		let value_ptr =
			CONTEXT.with(|context| unsafe { context.values.borrow_mut().alloc(initial_value) });
		let value = SignalValue::new(value_ptr as *mut c_void);
		let state = SignalState { value, subscribers: HashSet::new() };
		let raw_ptr = CONTEXT.with(|context| unsafe { context.signals.borrow_mut().alloc() });
		let state_ptr = raw_ptr as *mut SignalState;
		unsafe { *state_ptr = state };

		Signal { state_ptr, phantom: PhantomData }
	}

	pub(crate) fn get_current_mounter() -> Option<Subscriber> {
		CONTEXT.with(|context| context.current_mounter)
	}
}

const _: () = {
	// Slab Allocator Size
	if size_of::<SignalState>() > 64 {
		panic!("SignalData should be =<64");
	}
	if size_of::<SubscriberState>() > 32 {
		panic!("ScopeState should be =<32")
	}
};
