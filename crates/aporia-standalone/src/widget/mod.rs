mod handle;
mod reservation;
mod r#type;

pub use handle::WidgetHandle;
pub use reservation::Reservation;
pub(crate) use r#type::WidgetType;

use aporia_core::geometry::{Constraint, Size};

pub trait Widget {
	fn layout(&mut self, constraint: Constraint) -> Size;
}
