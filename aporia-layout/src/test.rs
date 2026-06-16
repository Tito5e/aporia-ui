use crate::geometry::{Constraint, Dimension, Padding};
use crate::layout::compute_layout;
use crate::node::{BoxLayoutNode, LayoutNode, LayoutNodeExt, ResolvedLayout};
use crate::style::Placement;
use aporia_core::geometry::Rect;
use std::ptr;

#[test]
fn box_in_box1() {
    let mut child_box_layout = Box::new(ResolvedLayout {
        rect: Rect::new(0f32, 0f32, 0f32, 0f32),
        is_x_changed: false,
        is_y_changed: false,
        is_width_changed: false,
        is_height_changed: false,
    });
    let mut child_box = Box::new(LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Percent(0.8)),
        height: Some(Dimension::Percent(0.8)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(0f32, 0f32, 0f32, 0f32),
        row_placement: Placement::Start,
        col_placement: Placement::Start,
        parent: ptr::null_mut(),
        child: ptr::null_mut(),
        resolved: &mut *child_box_layout,
    }));
    let mut parent_box_layout = Box::new(ResolvedLayout {
        rect: Rect::new(0f32, 0f32, 0f32, 0f32),
        is_x_changed: false,
        is_y_changed: false,
        is_width_changed: false,
        is_height_changed: false,
    });
    let mut parent_box = Box::new(LayoutNode::Box(BoxLayoutNode {
        width: Some(Dimension::Percent(1f32)),
        height: Some(Dimension::Percent(1f32)),
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        padding: Padding::tblr(10f32, 10f32, 10f32, 10f32),
        row_placement: Placement::Start,
        col_placement: Placement::End,
        parent: ptr::null_mut(),
        child: &mut *child_box,
        resolved: &mut *parent_box_layout,
    }));
    compute_layout(&mut *parent_box, Constraint::new(1920f32, 1080f32));
    assert_eq!(child_box.resolved_mut().rect.width, 1520f32);
    assert_eq!(child_box.resolved_mut().rect.height, 848f32);
    assert_eq!(child_box.resolved_mut().rect.x, 10f32);
    assert_eq!(child_box.resolved_mut().rect.y, 1080f32 - 848f32 - 10f32);
}
