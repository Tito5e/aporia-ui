use crate::geometry::{Constraint, Dimension, Padding};
use crate::node::{
	BoxLayoutNode, FlexChildLink, FlexLayoutNode, GridChildLink, GridLayoutNode, LayoutNode,
	RatioLayoutNode,
};
use crate::style::{Direction, FlexPlacement, Placement, RatioMode, WrapMode};
use std::ptr;

#[test]
fn box_in_box_alignment_start() {
	let child_res = create_resolved();
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
		resolved: child_res,
	});

	let parent_res = create_resolved();
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
		resolved: parent_res,
	});

	compute_layout(&mut parent, Constraint::new(200.0, 200.0));

	assert_eq!(child.resolved().rect.x, 0.0);
	assert_eq!(child.resolved().rect.y, 0.0);
}

#[test]
fn box_in_box_alignment_center_with_padding() {
	let child_res = create_resolved();
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
		resolved: child_res,
	});

	let parent_res = create_resolved();
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
		resolved: parent_res,
	});

	compute_layout(&mut parent, Constraint::new(200.0, 200.0));
	assert_eq!(child.resolved().rect.x, 50.0);
	assert_eq!(child.resolved().rect.y, 50.0);
}

#[test]
fn box_in_box_alignment_end() {
	let child_res = create_resolved();
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
		resolved: child_res,
	});

	let parent_res = create_resolved();
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
		resolved: parent_res,
	});

	compute_layout(&mut parent, Constraint::new(200.0, 200.0));
	assert_eq!(child.resolved().rect.x, 100.0);
	assert_eq!(child.resolved().rect.y, 100.0);
}

#[test]
fn box_in_ratio_width_mode() {
	let child_res = create_resolved();
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
		resolved: child_res,
	});

	let parent_res = create_resolved();
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
		resolved: parent_res,
	});

	compute_layout(&mut parent, Constraint::new(200.0, 200.0));

	assert_eq!(child.resolved().rect.width, 200.0);
	assert_eq!(child.resolved().rect.height, 100.0);
	assert_eq!(child.resolved().rect.x, 0.0);
	assert_eq!(child.resolved().rect.y, 50.0);
}

#[test]
fn ratio_in_box_height_mode_center() {
	let child_res = create_resolved();
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
		resolved: child_res,
	});

	let parent_res = create_resolved();
	let mut parent = LayoutNode::Ratio(RatioLayoutNode {
		mode: RatioMode::Height,
		ratio: 0.5,
		max_width: None,
		max_height: None,
		padding: Padding::tblr(0., 0., 0., 0.),
		row_placement: Placement::Center,
		col_placement: Placement::Center,
		parent: ptr::null_mut(),
		child: &mut child,
		resolved: parent_res,
	});

	compute_layout(&mut parent, Constraint::new(200.0, 200.0));

	assert_eq!(parent.resolved().rect.width, 100.0);
	assert_eq!(parent.resolved().rect.height, 200.0);
	assert_eq!(child.resolved().rect.x, 25.0);
	assert_eq!(child.resolved().rect.y, 75.0);
}

#[test]
fn box_min_max_constraints() {
	let child_res = create_resolved();
	let child = BoxLayoutNode {
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
		resolved: child_res,
	};

	let parent_res = create_resolved();
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
		child: &mut LayoutNode::Box(child),
		resolved: parent_res,
	});

	compute_layout(&mut parent, Constraint::new(500.0, 500.0));

	assert_eq!(parent.resolved().rect.width, 150.0);
	assert_eq!(parent.resolved().rect.height, 150.0);

	if let LayoutNode::Box(p) = &mut parent {
		if let LayoutNode::Box(c) = unsafe { &mut *p.child } {
			c.width = Some(Dimension::Px(300.0));
			c.height = Some(Dimension::Px(300.0));
		}
	}
	compute_layout(&mut parent, Constraint::new(500.0, 500.0));

	assert_eq!(parent.resolved().rect.width, 180.0);
	assert_eq!(parent.resolved().rect.height, 180.0);
}

#[test]
fn flex_row_no_wrap_alignment() {
	let c1_res = create_resolved();
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
		resolved: c1_res,
	});

	let c2_res = create_resolved();
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
		resolved: c2_res,
	});

	let mut link2 = FlexChildLink {
		child: &mut c2,
		prev: ptr::null_mut(),
		next: ptr::null_mut(),
		is_eol: true,
	};
	let mut link1 =
		FlexChildLink { child: &mut c1, prev: ptr::null_mut(), next: &mut link2, is_eol: false };
	link2.prev = &mut link1;

	let parent_res = create_resolved();
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
		resolved: parent_res,
		cross_axis_gap_between: 0f32,
		cross_axis_gap_edge: 0f32,
		line_placement: Placement::Start,
	});

	compute_layout(&mut parent, Constraint::new(200.0, 100.0));

	assert_eq!(c1.resolved().rect.x, 45.0);
	assert_eq!(c1.resolved().rect.y, 25.0);
	assert_eq!(c2.resolved().rect.x, 105.0);
	assert_eq!(c2.resolved().rect.y, 25.0);
}

#[test]
fn flex_column_wrap_alignment() {
	let c1_res = create_resolved();
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
		resolved: c1_res,
	});

	let c2_res = create_resolved();
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
		resolved: c2_res,
	});

	let mut link2 = FlexChildLink {
		child: &mut c2,
		prev: ptr::null_mut(),
		next: ptr::null_mut(),
		is_eol: true,
	};
	let mut link1 =
		FlexChildLink { child: &mut c1, prev: ptr::null_mut(), next: &mut link2, is_eol: false };
	link2.prev = &mut link1;

	let parent_res = create_resolved();
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
		resolved: parent_res,
		line_placement: Placement::Start,
		cross_axis_gap_edge: 0f32,
		cross_axis_gap_between: 0f32,
	});

	compute_layout(&mut parent, Constraint::new(200.0, 200.0));

	assert_eq!(c1.resolved().rect.x, 0.0);
	assert_eq!(c1.resolved().rect.y, 0.0);
	assert_eq!(c2.resolved().rect.x, 60.0);
	assert_eq!(c2.resolved().rect.y, 0.0);
}

#[test]
fn grid_alignment() {
	let c1_res = create_resolved();
	let mut c1 = LayoutNode::Box(BoxLayoutNode {
		width: Some(Dimension::Percent(0.5)),
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
		resolved: c1_res,
	});

	let c2_res = create_resolved();
	let mut c2 = LayoutNode::Box(BoxLayoutNode {
		width: Some(Dimension::Px(90.0)),
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
		resolved: c2_res,
	});

	let mut link2 = GridChildLink {
		child: &mut c2,
		row_placement: Placement::Start,
		col_placement: Placement::End,
		prev: ptr::null_mut(),
		next: ptr::null_mut(),
	};
	let mut link1 = GridChildLink {
		child: &mut c1,
		row_placement: Placement::Start,
		col_placement: Placement::Start,
		prev: ptr::null_mut(),
		next: &mut link2,
	};
	link2.prev = &mut link1;

	let parent_res = create_resolved();
	let mut parent = LayoutNode::Grid(GridLayoutNode {
		width: Some(Dimension::Percent(1.0)),
		height: Some(Dimension::Percent(1.0)),
		min_width: None,
		min_height: None,
		max_width: None,
		max_height: None,
		padding: Padding::tblr(0., 0., 0., 0.),
		direction: Direction::Row,
		wrap_size: 2,
		row_placement: FlexPlacement::Start,
		col_placement: FlexPlacement::Start,
		row_gap: 10.0,
		col_gap: 10.0,
		parent: ptr::null_mut(),
		child_head: &mut link1,
		resolved: parent_res,
		item_count: 2,
		row_axis_sizes: vec![],
		col_axis_sizes: vec![],
	});

	compute_layout(&mut parent, Constraint::new(200.0, 200.0));

	assert_eq!(c1.resolved().rect.width, 100.0);
	assert_eq!(c1.resolved().rect.height, 200.0);
	assert_eq!(c1.resolved().rect.x, 0.0);
	assert_eq!(c1.resolved().rect.y, 0.0);
	assert_eq!(c2.resolved().rect.x, 110.0);
	assert_eq!(c2.resolved().rect.y, 150.0);
}
