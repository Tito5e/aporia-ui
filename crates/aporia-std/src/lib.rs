mod block;
//mod ratio;

use aporia_core::geometry::Constraint;
use aporia_core::geometry::Size;
pub use block::Block;
//pub use ratio::Ratio;
//pub use ratio::RatioMode;

pub(crate) trait Measure {
	fn measure(&self, constraint: Constraint) -> Size;
}

pub(crate) trait IntrinsicWidth {
	fn intrinsic_width(&self, height: f32) -> f32;
}

pub(crate) trait IntrinsicHeight {
	fn intrinsic_height(&self, width: f32) -> f32;
}
