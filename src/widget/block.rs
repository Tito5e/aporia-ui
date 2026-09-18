use crate::{
	component::{Builder, NoChild, WidgetHandle},
	core::geometry::{Constraint, Dimension, Padding, Placement, Size},
	reactivity::context::Context,
	widget::Widget,
};

pub struct Block<T: Builder> {
	row_placement: Placement,
	col_placement: Placement,

	padding: Padding,

	width: Dimension,
	height: Dimension,
	min_width: Option<Dimension>,
	min_height: Option<Dimension>,
	max_width: Option<Dimension>,
	max_height: Option<Dimension>,

	child: Option<T>,
}

impl Block<NoChild> {
	#[inline]
	pub fn new() -> Self {
		// TODO: Defaultを用いた実装に切り替える
		Self {
			row_placement: Placement::Start,
			col_placement: Placement::Start,

			padding: Padding { top: 0.0, bottom: 0.0, left: 0.0, right: 0.0 },
			width: Dimension { percent: 0.0, px: 0.0 },
			height: Dimension { percent: 0.0, px: 0.0 },
			min_width: None,
			min_height: None,
			max_width: None,
			max_height: None,

			child: None,
		}
	}
}

impl<T: Builder> Block<T> {
	#[inline]
	pub fn child<C: Builder>(self, child: C) -> Block<C> {
		Block {
			row_placement: self.row_placement,
			col_placement: self.col_placement,

			padding: self.padding,
			width: self.width,
			height: self.height,
			min_width: self.min_width,
			min_height: self.min_height,
			max_width: self.max_width,
			max_height: self.max_height,

			child: Some(child),
		}
	}
}

impl<T: Builder> Builder for Block<T> {
	fn build(self) -> WidgetHandle {
		let child = self.child.map(|child| child.build());
		let block_data = BlockData {
			row_placement: self.row_placement,
			col_placement: self.col_placement,
			padding: self.padding,
			width: self.width,
			height: self.height,
			min_width: self.min_width,
			min_height: self.min_height,
			max_width: self.max_width,
			max_height: self.max_height,

			child,
		};

		Context::allocate_widget(block_data)
	}
}

// TODO: 制約のOptionをbitflags化し、構造体サイズを削減する
pub(crate) struct BlockData {
	pub row_placement: Placement,
	pub col_placement: Placement,

	pub padding: Padding,

	pub width: Dimension,
	pub height: Dimension,
	pub min_width: Option<Dimension>,
	pub min_height: Option<Dimension>,
	pub max_width: Option<Dimension>,
	pub max_height: Option<Dimension>,

	pub child: Option<WidgetHandle>,
}

impl Widget for BlockData {
	fn layout(&mut self, constraint: Constraint) -> Size {
		// TODO: 制約を実装する
		let self_width = self.width.resolve(constraint.width);
		let self_height = self.height.resolve(constraint.height);

		let self_x = match constraint.row_placement {
			Placement::Start => constraint.x,
			Placement::End => constraint.x + constraint.width - self_width,
			Placement::Center => constraint.x + (constraint.width - self_width) / 2.0f32,
		};
		let self_y = match constraint.col_placement {
			Placement::Start => constraint.y,
			Placement::End => constraint.y + constraint.height - self_height,
			Placement::Center => constraint.y + (constraint.height - self_height) / 2.0f32,
		};

		let child_x = self_x + self.padding.left;
		let child_y = self_y + self.padding.top;

		if let Some(ref mut child) = self.child {
			child.layout(Constraint::new(
				child_x,
				child_y,
				self_width - self.padding.left - self.padding.right,
				self_height - self.padding.top - self.padding.bottom,
				self.row_placement,
				self.col_placement,
			));
		}

		Size::new(self_width, self_height)
	}
}
