use crate::geometry::{Constraint, Dimension, Padding};
use crate::layout::compute_layout;
use crate::node::{
    BoxLayoutNode, FlexChildLink, FlexLayoutNode, LayoutNode, LayoutNodeExt, RatioLayoutNode,
    ResolvedLayout,
};
use crate::style::{Direction, FlexPlacement, Placement, RatioMode, WrapMode};
use aporia_core::geometry::Rect;
use std::ptr;

// Helper to create a basic ResolvedLayout
fn create_resolved() -> ResolvedLayout {
    ResolvedLayout {
        rect: Rect::new(0f32, 0f32, 0f32, 0f32),
        is_x_changed: false,
        is_y_changed: false,
        is_width_changed: false,
        is_height_changed: false,
    }
}

// === Box Parent -> Box Child Combinations ===

#[test]
fn box_in_box_alignment_start() {
    let mut child_res = create_resolved();
    let mut child = LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Px(100.0)),
        height: Some(Dimension::Px(100.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: ptr::null_mut(),
        resolved: &mut child_res,
    });

    let mut parent_res = create_resolved();
    let mut parent = LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Px(200.0)),
        height: Some(Dimension::Px(200.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: &mut child,
        resolved: &mut parent_res,
    });

    compute_layout(&mut parent, Constraint::new(200.0, 200.0));
    assert_eq!(child_res.rect.x, 0.0);
    assert_eq!(child_res.rect.y, 0.0);
}

#[test]
fn box_in_box_alignment_center_with_padding() {
    let mut child_res = create_resolved();
    let mut child = LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Px(100.0)),
        height: Some(Dimension::Px(100.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: ptr::null_mut(),
        resolved: &mut child_res,
    });

    let mut parent_res = create_resolved();
    let mut parent = LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Px(200.0)),
        height: Some(Dimension::Px(200.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(10., 10., 10., 10.),
        row_placement: Placement::Center,
        col_placement: Placement::Center,
        parent: ptr::null_mut(),
        child: &mut child,
        resolved: &mut parent_res,
    });

    compute_layout(&mut parent, Constraint::new(200.0, 200.0));
    // (200 - 100 - 10*2) / 2 = 40. local_offset=40, global_x = 10 + 40 = 50.
    assert_eq!(child_res.rect.x, 50.0);
    assert_eq!(child_res.rect.y, 50.0);
}

#[test]
fn box_in_box_alignment_end() {
    let mut child_res = create_resolved();
    let mut child = LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Px(100.0)),
        height: Some(Dimension::Px(100.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: ptr::null_mut(),
        resolved: &mut child_res,
    });

    let mut parent_res = create_resolved();
    let mut parent = LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Px(200.0)),
        height: Some(Dimension::Px(200.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::End,
        col_placement: Placement::End,
        parent: ptr::null_mut(),
        child: &mut child,
        resolved: &mut parent_res,
    });

    compute_layout(&mut parent, Constraint::new(200.0, 200.0));
    assert_eq!(child_res.rect.x, 100.0);
    assert_eq!(child_res.rect.y, 100.0);
}

// === Box Parent -> Ratio Child Combinations ===

#[test]
fn box_in_ratio_width_mode() {
    let mut child_res = create_resolved();
    let mut child = LayoutNode::Ratio(RatioLayoutNode {
        mode: RatioMode::Width,
        ratio: 2.0, // width is twice height
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: ptr::null_mut(),
        resolved: &mut child_res,
    });

    let mut parent_res = create_resolved();
    let mut parent = LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Px(200.0)),
        height: Some(Dimension::Px(200.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Center,
        col_placement: Placement::Center,
        parent: ptr::null_mut(),
        child: &mut child,
        resolved: &mut parent_res,
    });

    compute_layout(&mut parent, Constraint::new(200.0, 200.0));
    // Child width = 200, child height = 200 / 2 = 100.
    // Center alignment in 200x200: x = 0, y = (200 - 100) / 2 = 50.
    assert_eq!(child_res.rect.width, 200.0);
    assert_eq!(child_res.rect.height, 100.0);
    assert_eq!(child_res.rect.x, 0.0);
    assert_eq!(child_res.rect.y, 50.0);
}

// === Ratio Parent -> Box Child Combinations ===

#[test]
fn ratio_in_box_height_mode_center() {
    let mut child_res = create_resolved();
    let mut child = LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Px(50.0)),
        height: Some(Dimension::Px(50.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: ptr::null_mut(),
        resolved: &mut child_res,
    });

    let mut parent_res = create_resolved();
    let mut parent = LayoutNode::Ratio(RatioLayoutNode {
        mode: RatioMode::Height,
        ratio: 0.5, // width is half of height
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Center,
        col_placement: Placement::Center,
        parent: ptr::null_mut(),
        child: &mut child,
        resolved: &mut parent_res,
    });

    compute_layout(&mut parent, Constraint::new(200.0, 200.0));
    // Parent height = 200, parent width = 200 * 0.5 = 100.
    // Child 50x50 centered in 100x200: x = (100 - 50)/2 = 25, y = (200 - 50)/2 = 75.
    assert_eq!(parent_res.rect.width, 100.0);
    assert_eq!(parent_res.rect.height, 200.0);
    assert_eq!(child_res.rect.x, 25.0);
    assert_eq!(child_res.rect.y, 75.0);
}

// === Box Constraints (Min/Max) ===

#[test]
fn box_min_max_constraints() {
    let mut child_res = create_resolved();
    let mut child = LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Percent(1.0)),
        height: Some(Dimension::Percent(1.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: ptr::null_mut(),
        resolved: &mut child_res,
    });

    let mut parent_res = create_resolved();
    let mut parent = LayoutNode::Box(BoxLayoutNode {
        width: None,
        height: None,
        min_width: Some(Dimension::Px(150.0)),
        min_height: Some(Dimension::Px(150.0)),
        max_width: Some(Dimension::Px(180.0)),
        max_height: Some(Dimension::Px(180.0)),
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: &mut child,
        resolved: &mut parent_res,
    });

    // Case 1: Child is small, parent should be min_size
    // (Note: Child Percent size resolves against parent size which is currently unknown during compute_size 
    // but in this implementation Box(None) fits child. If child is Box(100.0), it's a bit circular.
    // Let's use a fixed size child for constraint testing)
    let mut child_fixed = BoxLayoutNode {
        width: Some(Dimension::Px(100.0)),
        height: Some(Dimension::Px(100.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: ptr::null_mut(),
        resolved: &mut child_res,
    };
    let mut parent_node = LayoutNode::Box(BoxLayoutNode {
        width: None,
        height: None,
        min_width: Some(Dimension::Px(150.0)),
        min_height: Some(Dimension::Px(150.0)),
        max_width: Some(Dimension::Px(180.0)),
        max_height: Some(Dimension::Px(180.0)),
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: &mut LayoutNode::Box(child_fixed),
        resolved: &mut parent_res,
    });

    compute_layout(&mut parent_node, Constraint::new(500.0, 500.0));
    assert_eq!(parent_res.rect.width, 150.0);
    assert_eq!(parent_res.rect.height, 150.0);

    // Case 2: Child is large, parent should be max_size
    if let LayoutNode::Box(p) = &mut parent_node {
        if let LayoutNode::Box(c) = unsafe { &mut *p.child } {
            c.width = Some(Dimension::Px(300.0));
            c.height = Some(Dimension::Px(300.0));
        }
    }
    compute_layout(&mut parent_node, Constraint::new(500.0, 500.0));
    assert_eq!(parent_res.rect.width, 180.0);
    assert_eq!(parent_res.rect.height, 180.0);
}

// === Flex Parent Combinations ===

#[test]
fn flex_row_no_wrap_alignment() {
    let mut c1_res = create_resolved();
    let mut c1 = LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Px(50.0)),
        height: Some(Dimension::Px(50.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: ptr::null_mut(),
        resolved: &mut c1_res,
    });

    let mut c2_res = create_resolved();
    let mut c2 = LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Px(50.0)),
        height: Some(Dimension::Px(50.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: ptr::null_mut(),
        resolved: &mut c2_res,
    });

    let mut link2 = FlexChildLink {
        child: &mut c2,
        prev: ptr::null_mut(),
        next: ptr::null_mut(),
        is_eol: false,
    };
    let mut link1 = FlexChildLink {
        child: &mut c1,
        prev: ptr::null_mut(),
        next: &mut link2,
        is_eol: false,
    };
    link2.prev = &mut link1;

    let mut parent_res = create_resolved();
    let mut parent = LayoutNode::Flex(FlexLayoutNode {
        width: Some(Dimension::Px(200.0)),
        height: Some(Dimension::Px(100.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        direction: Direction::Row,
        wrap_mode: WrapMode::NoWrap,
        row_placement: FlexPlacement::Center,
        col_placement: FlexPlacement::Center,
        row_gap: 10.0,
        col_gap: 0.0,
        parent: ptr::null_mut(),
        child_head: &mut link1,
        resolved: &mut parent_res,
    });

    compute_layout(&mut parent, Constraint::new(200.0, 100.0));
    // Total children width = 50 + 10 + 50 = 110.
    // Center alignment in 200: start_x = (200 - 110) / 2 = 45.
    // Center alignment in 100: y = (100 - 50) / 2 = 25.
    // c1.x = 45, c2.x = 45 + 50 + 10 = 105.
    // Note: Currently fails as compute_flex_offset is empty.
    assert_eq!(c1_res.rect.x, 45.0);
    assert_eq!(c1_res.rect.y, 25.0);
    assert_eq!(c2_res.rect.x, 105.0);
    assert_eq!(c2_res.rect.y, 25.0);
}

#[test]
fn flex_column_wrap_alignment() {
    let mut c1_res = create_resolved();
    let mut c1 = LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Px(50.0)),
        height: Some(Dimension::Px(150.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: ptr::null_mut(),
        resolved: &mut c1_res,
    });

    let mut c2_res = create_resolved();
    let mut c2 = LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Px(50.0)),
        height: Some(Dimension::Px(150.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: ptr::null_mut(),
        resolved: &mut c2_res,
    });

    let mut link2 = FlexChildLink {
        child: &mut c2,
        prev: ptr::null_mut(),
        next: ptr::null_mut(),
        is_eol: false,
    };
    let mut link1 = FlexChildLink {
        child: &mut c1,
        prev: ptr::null_mut(),
        next: &mut link2,
        is_eol: false,
    };
    link2.prev = &mut link1;

    let mut parent_res = create_resolved();
    let mut parent = LayoutNode::Flex(FlexLayoutNode {
        width: Some(Dimension::Px(200.0)),
        height: Some(Dimension::Px(200.0)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0., 0., 0., 0.),
        direction: Direction::Column,
        wrap_mode: WrapMode::Wrap,
        row_placement: FlexPlacement::Start,
        col_placement: FlexPlacement::Start,
        row_gap: 10.0,
        col_gap: 0.0,
        parent: ptr::null_mut(),
        child_head: &mut link1,
        resolved: &mut parent_res,
    });

    compute_layout(&mut parent, Constraint::new(200.0, 200.0));
    // Column wrap: c1 is 150h, fits. c2 is 150h, 150+150 > 200, so wraps to next column.
    // Column 1: c1 at x=0, y=0.
    // Column 2: c2 at x=50+10=60, y=0.
    assert_eq!(c1_res.rect.x, 0.0);
    assert_eq!(c1_res.rect.y, 0.0);
    assert_eq!(c2_res.rect.x, 60.0);
    assert_eq!(c2_res.rect.y, 0.0);
}
