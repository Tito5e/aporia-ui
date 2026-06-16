use crate::geometry::{Dimension, Padding};
use crate::style::{Direction, FlexPlacement, Placement, RatioMode, WrapMode};
use aporia_core::geometry::Rect;

pub enum LayoutNode {
    Box(BoxLayoutNode),
    Ratio(RatioLayoutNode),
    Flex(FlexLayoutNode),
}

impl LayoutNode {
    pub fn clear_dirty(&self) {
        match self {
            LayoutNode::Box(node) => {
                let self_resolved = unsafe { &mut *node.resolved };
                self_resolved.is_width_changed = false;
                self_resolved.is_height_changed = false;
                self_resolved.is_x_changed = false;
                self_resolved.is_y_changed = false;
            }
            LayoutNode::Ratio(node) => {
                let self_resolved = unsafe { &mut *node.resolved };
                self_resolved.is_width_changed = false;
                self_resolved.is_height_changed = false;
                self_resolved.is_x_changed = false;
                self_resolved.is_y_changed = false;
            }
            LayoutNode::Flex(node) => {
                let self_resolved = unsafe { &mut *node.resolved };
                self_resolved.is_width_changed = false;
                self_resolved.is_height_changed = false;
                self_resolved.is_x_changed = false;
                self_resolved.is_y_changed = false;
            }
        }
    }
}

pub(crate) trait LayoutNodeExt {
    fn resolved_mut(&self) -> &mut ResolvedLayout;
}

impl LayoutNodeExt for LayoutNode {
    #[inline(always)]
    fn resolved_mut(&self) -> &mut ResolvedLayout {
        match self {
            LayoutNode::Box(node) => unsafe { &mut *node.resolved },
            LayoutNode::Ratio(node) => unsafe { &mut *node.resolved },
            LayoutNode::Flex(node) => unsafe { &mut *node.resolved },
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

    pub resolved: *mut ResolvedLayout,
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

    pub resolved: *mut ResolvedLayout,
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
    pub row_gap: f32,
    pub col_gap: f32,

    pub parent: *mut LayoutNode,
    pub child_head: *mut FlexChildLink,

    pub resolved: *mut ResolvedLayout,
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
    pub row_gap: f32,
    pub col_gap: f32,
    pub default_row_track: Dimension,
    pub default_col_track: Dimension,

    pub parent: *mut LayoutNode,
    pub child_head: *mut GridChildLink,

    pub resolved: *mut ResolvedLayout,
}

pub struct ResolvedLayout {
    pub rect: Rect,
    pub is_x_changed: bool,
    pub is_y_changed: bool,
    pub is_width_changed: bool,
    pub is_height_changed: bool,
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
