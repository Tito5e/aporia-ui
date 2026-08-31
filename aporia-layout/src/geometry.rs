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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraint {
	pub max_width: f32,
	pub max_height: f32,
}

impl Constraint {
	#[inline(always)]
	pub fn new(max_width: f32, max_height: f32) -> Self {
		Self { max_width, max_height }
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Preferred {
	pub width: Option<f32>,
	pub height: Option<f32>,
}

impl Preferred {
	#[inline(always)]
	pub fn new(width: Option<f32>, height: Option<f32>) -> Self {
		Self { width, height }
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
	Percent(f32),
	Px(f32),
}

impl Dimension {
	pub fn resolve(&self, constraint: f32) -> f32 {
		match self {
			Dimension::Percent(percent) => constraint * percent,
			Dimension::Px(pixel) => *pixel,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridDimension {
	Percent(f32),
	Px(f32),
	Fr(f32),
}

impl GridDimension {
	pub fn resolve(&self, constraint: f32, fr_unit: Option<f32>) -> f32 {
		match self {
			GridDimension::Percent(percent) => constraint * percent,
			GridDimension::Px(pixel) => *pixel,

			GridDimension::Fr(fr) => {
				if let Some(fr_unit) = fr_unit {
					fr_unit * fr
				} else {
					constraint * fr
				}
			}
		}
	}

	pub fn is_fr(&self) -> bool {
		match self {
			GridDimension::Fr(_) => true,
			_ => false,
		}
	}
}

pub trait DimensionResolver {
	fn to_constraint(&self, min: Option<Dimension>, max: Option<Dimension>, constraint: f32)
	-> f32;
}

impl DimensionResolver for Option<GridDimension> {
	fn to_constraint(
		&self,
		min: Option<Dimension>,
		max: Option<Dimension>,
		constraint: f32,
	) -> f32 {
		match self {
			Some(dimension) => {
				// Ownモード: 自身で宣言したサイズを優先しつつ、各種制約に従う
				let mut preferred = match dimension {
					GridDimension::Percent(percent) => constraint * percent,
					GridDimension::Px(pixel) => *pixel,
					GridDimension::Fr(fr) => constraint * fr,
				};

				// 優先度としてはminの方を強くする
				if let Some(max) = max {
					preferred = max.resolve(constraint).min(preferred);
				}
				if let Some(min) = min {
					preferred = min.resolve(constraint).max(preferred);
				}

				preferred
			}
			None => {
				// Fitモード: 自身ではサイズの宣言を持たず、子要素のサイズを優先しつつ、各種制約に従う
				if let Some(max) = max { max.resolve(constraint) } else { constraint }
			}
		}
	}
}

impl From<Dimension> for GridDimension {
	fn from(value: Dimension) -> Self {
		match value {
			Dimension::Percent(percent) => GridDimension::Percent(percent),
			Dimension::Px(px) => GridDimension::Px(px),
		}
	}
}
