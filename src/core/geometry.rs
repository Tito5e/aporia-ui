use std::ops;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
	pub width: f32,
	pub height: f32,
}

impl Size {
	#[inline(always)]
	pub fn new(width: f32, height: f32) -> Self {
		Self { width, height }
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
	pub x: f32,
	pub y: f32,
	pub width: f32,
	pub height: f32,
}

impl Rect {
	#[inline(always)]
	pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
		Self { x, y, width, height }
	}

	#[inline(always)]
	pub fn size(&self) -> Size {
		Size::new(self.width, self.height)
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraint {
	pub x: f32,
	pub y: f32,
	pub width: f32,
	pub height: f32,

	pub row_placement: Placement,
	pub col_placement: Placement,
}

impl Constraint {
	#[inline(always)]
	pub fn new(
		x: f32,
		y: f32,
		width: f32,
		height: f32,
		row_placement: Placement,
		col_placement: Placement,
	) -> Self {
		Self { x, y, width, height, row_placement, col_placement }
	}
}

/// 要素の配置を表現します
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
	/// 左、もしくは上に配置します
	Start,
	/// 右、もしくは下に配置します
	End,
	/// 中央に配置します
	Center,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dimension {
	pub percent: f32,
	pub px: f32,
}

impl Dimension {
	#[inline(always)]
	pub(crate) fn resolve(&self, constraint: f32) -> f32 {
		constraint * self.percent + self.px
	}

	#[inline(always)]
	pub fn percent(value: f32) -> Self {
		Self { percent: value, px: 0.0 }
	}

	#[inline(always)]
	pub fn px(value: f32) -> Self {
		Self { percent: 0.0, px: value }
	}
}

impl ops::Add<Dimension> for Dimension {
	type Output = Dimension;

	fn add(self, other: Dimension) -> Dimension {
		Dimension { percent: self.percent + other.percent, px: self.px + other.px }
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridDimension {
	pub percent: f32,
	pub px: f32,
	pub fr: f32,
}

impl GridDimension {
	#[inline(always)]
	pub fn measure(&self, constraint: f32) -> f32 {
		constraint * self.percent + self.px
	}

	#[inline(always)]
	pub fn resolve(&self, constraint: f32, fr_unit: f32) -> f32 {
		constraint * self.percent + self.px + fr_unit * self.fr
	}

	#[inline(always)]
	pub fn has_fr(&self) -> bool {
		self.fr != 0.0f32
	}
}

/// 要素のpaddingを表現します
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Padding {
	/// padding-topに該当します
	pub top: f32,
	/// padding-bottomに該当します
	pub bottom: f32,
	/// padding-leftに該当します
	pub left: f32,
	/// padding-rightに該当します
	pub right: f32,
}

impl Padding {
	#[inline(always)]
	pub fn tblr(top: f32, bottom: f32, left: f32, right: f32) -> Self {
		Self { top, bottom, left, right }
	}
}
