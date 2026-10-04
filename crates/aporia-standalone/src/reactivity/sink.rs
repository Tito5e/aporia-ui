use crate::reactivity::effect::EffectHeader;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SinkHeader {
	pub mark: unsafe fn(*mut SinkHeader, *mut EffectHeader) -> u32,

	pub cancel: unsafe fn(*mut SinkHeader, u32),
}
