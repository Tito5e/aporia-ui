use crate::context::StandaloneCx;

pub struct ReconcileCx<'a> {
	pub(crate) cx: &'a mut StandaloneCx,
}
