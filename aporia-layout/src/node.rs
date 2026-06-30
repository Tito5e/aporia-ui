use crate::geometry::{Dimension, Padding};
use crate::style::{Direction, FlexPlacement, Placement, RatioMode, WrapMode};
use aporia_core::geometry::Rect;

pub enum LayoutNode {
    Box(BoxLayoutNode),
    Ratio(RatioLayoutNode),
    Flex(FlexLayoutNode),
    Grid(GridLayoutNode),
}

impl LayoutNode {
    #[inline(always)]
    pub fn clear_dirty(&mut self) {
        let self_resolved = self.resolved_mut();
        self_resolved.is_width_changed = false;
        self_resolved.is_height_changed = false;
        self_resolved.is_x_changed = false;
        self_resolved.is_y_changed = false;
    }
}

pub(crate) trait LayoutNodeExt {
    fn resolved_mut(&mut self) -> &mut ResolvedLayout;
    fn resolved(&self) -> &ResolvedLayout;
}

impl LayoutNodeExt for LayoutNode {
    #[inline(always)]
    fn resolved_mut(&mut self) -> &mut ResolvedLayout {
        match self {
            LayoutNode::Box(node) => &mut node.resolved,
            LayoutNode::Ratio(node) => &mut node.resolved,
            LayoutNode::Flex(node) => &mut node.resolved,
            LayoutNode::Grid(node) => &mut node.resolved,
        }
    }

    #[inline(always)]
    fn resolved(&self) -> &ResolvedLayout {
        match self {
            LayoutNode::Box(node) => &node.resolved,
            LayoutNode::Ratio(node) => &node.resolved,
            LayoutNode::Flex(node) => &node.resolved,
            LayoutNode::Grid(node) => &node.resolved,
        }
    }
}

pub struct BoxLayoutNode {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub min_width: Option<Dimension>,
    pub min_height: Option<Dimension>,
    pub max_width: Option<Dimension>,
    pub max_height: Option<Dimension>,

    pub padding: Padding,

    pub row_placement: Placement,
    pub col_placement: Placement,

    pub parent: *mut LayoutNode,
    pub child: *mut LayoutNode,

    pub resolved: ResolvedLayout,
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
    pub child: *mut LayoutNode,

    pub resolved: ResolvedLayout,
}

#[test]
pub fn layout_node_size() {
    println!("{}", size_of::<BoxLayoutNode>())
}

pub struct FlexLayoutNode {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
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
    pub child_head: *mut FlexChildLink,

    pub resolved: ResolvedLayout,
    // This is cache !! DON'T USE !!
    pub cross_axis_gap_edge: f32,
    pub cross_axis_gap_between: f32,
}

pub struct GridLayoutNode {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
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
    pub child_head: *mut GridChildLink,

    pub resolved: ResolvedLayout,
    // This is cache !! DON'T UPDATE MANUALLY !!
    pub item_count: usize,
    pub row_axis_sizes: Vec<f32>,
    pub col_axis_sizes: Vec<f32>,
}

pub struct ResolvedLayout {
    pub rect: Rect,
    pub is_x_changed: bool,
    pub is_y_changed: bool,
    pub is_width_changed: bool,
    pub is_height_changed: bool,
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

pub struct FlexChildLink {
    pub child: *mut LayoutNode,

    pub prev: *mut FlexChildLink,
    pub next: *mut FlexChildLink,

    pub is_eol: bool,
}

pub struct GridChildLink {
    pub child: *mut LayoutNode,

    pub row_placement: Placement,
    pub col_placement: Placement,

    pub prev: *mut GridChildLink,
    pub next: *mut GridChildLink,
}
