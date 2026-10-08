use aporia_core::geometry::{Dimension, Padding, Placement};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatioMode {
	Width,
	Height,
	Fit,
	Fill,
}

pub struct Ratio<S, T> {
	pub row_placement: Placement,
	pub col_placement: Placement,

	pub padding: Padding,

	pub mode: RatioMode,
	pub ratio: f32,
	pub max_width: Option<Dimension>,
	pub max_height: Option<Dimension>,

	pub child: Option<Box<dyn Widget>>,
}

impl Widget for Ratio {
	fn layout(&mut self, constraint: Constraint) -> Size {
		todo!()
	}
}
