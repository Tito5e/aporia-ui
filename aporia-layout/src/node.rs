use crate::geometry::{Dimension, GridDimension, Padding};
use crate::style::{Direction, FlexPlacement, Placement, RatioMode, WrapMode};
use aporia_core::geometry::Rect;

pub(crate) enum LayoutNode {
	Box(BoxLayoutNode),
	Ratio(RatioLayoutNode),
	Flex(FlexLayoutNode),
	Grid(GridLayoutNode),
}

impl LayoutNode {
	#[inline(always)]
	pub(crate) fn resolved_mut(&mut self) -> &mut Rect {
		match self {
			LayoutNode::Box(node) => &mut node.resolved,
			LayoutNode::Ratio(node) => &mut node.resolved,
			LayoutNode::Flex(node) => &mut node.resolved,
			LayoutNode::Grid(node) => &mut node.resolved,
		}
	}

	#[inline(always)]
	pub(crate) fn resolved(&self) -> &Rect {
		match self {
			LayoutNode::Box(node) => &node.resolved,
			LayoutNode::Ratio(node) => &node.resolved,
			LayoutNode::Flex(node) => &node.resolved,
			LayoutNode::Grid(node) => &node.resolved,
		}
	}

	#[inline(always)]
	pub(crate) fn clear_dirty(&mut self) {
		match self {
			LayoutNode::Box(node) => node.dirty = false,
			LayoutNode::Ratio(node) => node.dirty = false,
			LayoutNode::Flex(node) => node.dirty = false,
			LayoutNode::Grid(node) => node.dirty = false,
		}
	}

	#[inline(always)]
	pub(crate) fn mark_dirty(&mut self) {
		match self {
			LayoutNode::Box(node) => node.dirty = true,
			LayoutNode::Ratio(node) => node.dirty = true,
			LayoutNode::Flex(node) => node.dirty = true,
			LayoutNode::Grid(node) => node.dirty = true,
		}
	}

	#[inline(always)]
	pub(crate) fn set_size(&mut self, width: f32, height: f32) {
		let self_resolved = self.resolved_mut();
		self_resolved.width = width;
		self_resolved.height = height;

		self.mark_dirty();
	}

	#[inline(always)]
	pub(crate) fn set_width(&mut self, width: f32) {
		let self_resolved = self.resolved_mut();
		self_resolved.width = width;

		self.mark_dirty();
	}

	#[inline(always)]
	pub(crate) fn set_height(&mut self, height: f32) {
		let self_resolved = self.resolved_mut();
		self_resolved.height = height;

		self.mark_dirty();
	}

	#[inline(always)]
	pub(crate) fn width(&self) -> Option<GridDimension> {
		match self {
			LayoutNode::Box(node) => node.width,
			LayoutNode::Ratio(node) => node.max_width.map(|max_width| max_width.into()),
			LayoutNode::Flex(node) => node.width,
			LayoutNode::Grid(node) => node.width,
		}
	}

	#[inline(always)]
	pub(crate) fn height(&self) -> Option<GridDimension> {
		match self {
			LayoutNode::Box(node) => node.height,
			LayoutNode::Ratio(node) => node.max_height.map(|max_height| max_height.into()),
			LayoutNode::Flex(node) => node.height,
			LayoutNode::Grid(node) => node.height,
		}
	}

	#[inline(always)]
	pub(crate) fn min_width(&self) -> Option<Dimension> {
		match self {
			LayoutNode::Box(node) => node.min_width,
			LayoutNode::Ratio(_) => None,
			LayoutNode::Flex(node) => node.min_width,
			LayoutNode::Grid(node) => node.min_width,
		}
	}

	#[inline(always)]
	pub(crate) fn min_height(&self) -> Option<Dimension> {
		match self {
			LayoutNode::Box(node) => node.min_height,
			LayoutNode::Ratio(_) => None,
			LayoutNode::Flex(node) => node.min_height,
			LayoutNode::Grid(node) => node.min_height,
		}
	}

	#[inline(always)]
	pub(crate) fn max_width(&self) -> Option<Dimension> {
		match self {
			LayoutNode::Box(node) => node.max_width,
			LayoutNode::Ratio(node) => node.max_width,
			LayoutNode::Flex(node) => node.max_width,
			LayoutNode::Grid(node) => node.max_width,
		}
	}

	#[inline(always)]
	pub(crate) fn max_height(&self) -> Option<Dimension> {
		match self {
			LayoutNode::Box(node) => node.max_height,
			LayoutNode::Ratio(node) => node.max_height,
			LayoutNode::Flex(node) => node.max_height,
			LayoutNode::Grid(node) => node.max_height,
		}
	}
}

pub struct BoxLayoutNode {
	pub width: Option<GridDimension>,
	pub height: Option<GridDimension>,

	pub min_width: Option<Dimension>,
	pub min_height: Option<Dimension>,
	pub max_width: Option<Dimension>,
	pub max_height: Option<Dimension>,

	pub padding: Padding,

	pub row_placement: Placement,
	pub col_placement: Placement,

	pub parent: *mut LayoutNode,
	pub child: BoxChildLink,

	pub resolved: Rect,
	pub dirty: bool,
}

pub struct RatioLayoutNode {
	pub mode: RatioMode,
	pub ratio: f32,
	pub max_width: Option<Dimension>,
	pub max_height: Option<Dimension>,
	pub padding: Padding,

	pub row_placement: Placement,
	pub col_placement: Placement,

	pub parent: *mut LayoutNode,
	pub child: RatioChildLink,

	pub resolved: Rect,
	pub dirty: bool,
}

#[test]
pub fn layout_node_size() {
	println!("{}", size_of::<BoxLayoutNode>())
}

pub struct FlexLayoutNode {
	pub width: Option<GridDimension>,
	pub height: Option<GridDimension>,

	pub min_width: Option<Dimension>,
	pub min_height: Option<Dimension>,
	pub max_width: Option<Dimension>,
	pub max_height: Option<Dimension>,

	pub padding: Padding,

	pub direction: Direction,
	pub wrap_mode: WrapMode,

	pub row_placement: FlexPlacement,
	pub col_placement: FlexPlacement,
	pub line_placement: Placement,
	pub row_gap: f32,
	pub col_gap: f32,

	pub parent: *mut LayoutNode,
	pub children: Vec<FlexChildLink>,

	pub resolved: Rect,
	pub dirty: bool,

	// This is cache !! DON'T USE !!
	pub cross_axis_gap_edge: f32,
	pub cross_axis_gap_between: f32,
}

pub struct GridLayoutNode {
	pub width: Option<GridDimension>,
	pub height: Option<GridDimension>,

	pub min_width: Option<Dimension>,
	pub min_height: Option<Dimension>,
	pub max_width: Option<Dimension>,
	pub max_height: Option<Dimension>,

	pub padding: Padding,

	pub direction: Direction,
	pub wrap_size: usize,

	pub row_placement: FlexPlacement,
	pub col_placement: FlexPlacement,
	pub row_gap: f32,
	pub col_gap: f32,

	pub parent: *mut LayoutNode,
	pub children: Vec<GridChildLink>,

	pub resolved: Rect,
	pub dirty: bool,

	// This is cache !! DON'T UPDATE MANUALLY !!
	pub item_count: usize,
	pub row_axis_sizes: Vec<f32>,
	pub col_axis_sizes: Vec<f32>,
}

const LAYOUT_THRESHOLD: f32 = 0.1;

impl ResolvedLayout {
	#[inline(always)]
	pub(crate) fn update_width(&mut self, width: f32) {
		if (self.rect.width - width).abs() > LAYOUT_THRESHOLD {
			self.is_width_changed = true;
			self.rect.width = width;
		}
	}

	#[inline(always)]
	pub(crate) fn update_height(&mut self, height: f32) {
		if (self.rect.height - height).abs() > LAYOUT_THRESHOLD {
			self.is_height_changed = true;
			self.rect.height = height;
		}
	}

	#[inline(always)]
	pub(crate) fn update_x(&mut self, x: f32) {
		if (self.rect.x - x).abs() > LAYOUT_THRESHOLD {
			self.is_x_changed = true;
			self.rect.x = x;
		}
	}

	#[inline(always)]
	pub(crate) fn update_y(&mut self, y: f32) {
		if (self.rect.y - y).abs() > LAYOUT_THRESHOLD {
			self.is_y_changed = true;
			self.rect.y = y;
		}
	}
}

pub struct BoxChildLink {
	pub ptr: *mut LayoutNode,
}

pub struct RatioChildLink {
	pub ptr: *mut LayoutNode,
}

pub struct FlexChildLink {
	pub ptr: *mut LayoutNode,
	pub is_split: bool,
}

pub struct GridChildLink {
	pub ptr: *mut LayoutNode,

	pub row_placement: Placement,
	pub col_placement: Placement,
}
