use crate::{application::StandaloneCx, widget::WidgetHandle};

pub trait Builder {
	#[doc(hidden)]
	fn build(self, cx: &mut StandaloneCx) -> WidgetHandle;
}
