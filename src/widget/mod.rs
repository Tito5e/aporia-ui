mod block;
mod ratio;

pub use block::Block;

use crate::core::geometry::{Constraint, Size};

pub(crate) trait Measure {
	fn measure(&self, constraint: Constraint) -> Size;
}

pub(crate) trait IntrinsicWidth {
	fn intrinsic_width(&self, height: f32) -> f32;
}

pub(crate) trait IntrinsicHeight {
	fn intrinsic_height(&self, width: f32) -> f32;
}

pub trait Widget {
	fn layout(&mut self, constraint: Constraint) -> Size;
}

pub trait Remountable {
	fn remount(&mut self);
}
