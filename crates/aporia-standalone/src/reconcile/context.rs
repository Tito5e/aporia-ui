use crate::application::StandaloneCx;

pub struct ReconcileCx<'a> {
	pub(crate) cx: &'a mut StandaloneCx,
}
