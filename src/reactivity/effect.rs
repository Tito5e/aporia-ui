use std::ptr::NonNull;

use crate::widget::Remountable;

pub(crate) struct EffectState {
	apply_stage: ApplyStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Effect(*mut EffectState);

pub(crate) enum ApplyStage {
	Remount,
	Logic,
	Render,
}
