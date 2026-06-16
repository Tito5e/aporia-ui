use crate::geometry::{Constraint, Dimension, Padding};
use crate::layout::compute_layout;
use crate::node::{
    BoxLayoutNode, LayoutNode, LayoutNodeExt, ResolvedLayout,
};
use crate::style::{Placement};
use aporia_core::geometry::Rect;
use std::ptr;

fn create_resolved() -> ResolvedLayout {
    ResolvedLayout {
        rect: Rect::new(0f32, 0f32, 0f32, 0f32),
        is_x_changed: false,
        is_y_changed: false,
        is_width_changed: false,
        is_height_changed: false,
    }
}

// Exhaustive combinations for Box in Box (Width only, as Height is symmetrical)
#[test]
fn box_in_box_exhaustive_width() {
    let placements = [Placement::Start, Placement::Center, Placement::End];
    let paddings = [0.0, 10.0];
    let child_widths = [50.0, 100.0, 150.0];
    let parent_widths = [200.0];

    for &rp in &placements {
        for &pad in &paddings {
            for &cw in &child_widths {
                for &pw in &parent_widths {
                    let mut child_res = create_resolved();
                    let mut child = LayoutNode::Box(BoxLayoutNode {
                        width: Some(Dimension::Px(cw)),
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
                    let mut parent = LayoutNode::Box(BoxLayoutNode {
                        width: Some(Dimension::Px(pw)),
                        height: Some(Dimension::Px(200.0)),
                        min_width: None,
                        min_height: None,
                        max_width: None,
                        max_height: None,
                        padding: Padding::tblr(0., 0., pad, pad), // symmetric padding for simplicity in loop
                        row_placement: rp,
                        col_placement: Placement::Start,
                        parent: ptr::null_mut(),
                        child: &mut child,
                        resolved: &mut parent_res,
                    });

                    compute_layout(&mut parent, Constraint::new(pw, 200.0));

                    // Expected global x calculation
                    let inner_w = pw - 2.0 * pad;
                    let local_x = match rp {
                        Placement::Start => 0.0,
                        Placement::Center => (inner_w - cw) / 2.0,
                        Placement::End => inner_w - cw,
                    };
                    let expected_x = pad + local_x;

                    assert!(
                        (child_res.rect.x - expected_x).abs() < 0.1,
                        "Failed: rp={:?}, pad={}, cw={}, pw={}. Expected x={}, got {}",
                        rp, pad, cw, pw, expected_x, child_res.rect.x
                    );
                }
            }
        }
    }
}
