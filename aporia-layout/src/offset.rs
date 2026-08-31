use crate::geometry::{Constraint, Dimension, Preferred};
use crate::node::{BoxLayoutNode, FlexLayoutNode, GridLayoutNode, LayoutNode, RatioLayoutNode};
use crate::style::{Direction, FlexPlacement, Placement, RatioMode, WrapMode};
use aporia_core::geometry::Size;

pub(crate) fn compute_offset(node: &mut LayoutNode, offset: Size) {
	match node {
		LayoutNode::Box(node) => {
			compute_box_offset(node, offset);
		}
		LayoutNode::Ratio(node) => {
			compute_ratio_offset(node, offset);
		}
		LayoutNode::Flex(node) => {
			compute_flex_offset(node, offset);
		}
		LayoutNode::Grid(node) => {
			compute_grid_offset(node, offset);
		}
	}
}

fn compute_box_offset(node: &mut BoxLayoutNode, offset: Size) {
	if !node.child.ptr.is_null() {
		let child_node = unsafe { &mut *node.child.ptr };
		let local_offset_x = match node.row_placement {
			Placement::Start => 0f32,
			Placement::End => {
				let width_margin = node.resolved.width
					- child_node.resolved().width
					- node.padding.left
					- node.padding.right;

				width_margin
			}
			Placement::Center => {
				let width_margin = node.resolved.width
					- child_node.resolved().width
					- node.padding.left
					- node.padding.right;

				width_margin / 2f32
			}
		};

		let local_offset_y =
			match node.col_placement {
				Placement::Start => 0f32,
				Placement::End => {
					let height_margin = node.resolved.height
						- child_node.resolved().height
						- node.padding.top - node.padding.bottom;

					height_margin
				}
				Placement::Center => {
					let height_margin = node.resolved.height
						- child_node.resolved().height
						- node.padding.top - node.padding.bottom;

					height_margin / 2f32
				}
			};

		child_node.resolved_mut().update_x(offset.width + local_offset_x + node.padding.left);
		child_node.resolved_mut().update_y(offset.height + local_offset_y + node.padding.top);

		compute_offset(
			child_node,
			Size::new(
				offset.width + local_offset_x + node.padding.left,
				offset.height + local_offset_y + node.padding.top,
			),
		)
	}
}

fn compute_ratio_offset(node: &mut RatioLayoutNode, offset: Size) {
	if !node.child.ptr.is_null() {
		let child_node = unsafe { &mut *node.child.ptr };

		let local_offset_x = match node.row_placement {
			Placement::Start => 0f32,
			Placement::End => {
				let width_margin = node.resolved.width
					- child_node.resolved().width
					- node.padding.left
					- node.padding.right;

				width_margin
			}
			Placement::Center => {
				let width_margin = node.resolved.width
					- child_node.resolved().width
					- node.padding.left
					- node.padding.right;

				width_margin / 2f32
			}
		};

		let local_offset_y =
			match node.col_placement {
				Placement::Start => 0f32,
				Placement::End => {
					let height_margin = node.resolved.height
						- child_node.resolved().height
						- node.padding.top - node.padding.bottom;

					height_margin
				}
				Placement::Center => {
					let height_margin = node.resolved.height
						- child_node.resolved().height
						- node.padding.top - node.padding.bottom;

					height_margin / 2f32
				}
			};

		child_node.resolved_mut().update_x(offset.width + local_offset_x + node.padding.left);
		child_node.resolved_mut().update_y(offset.height + local_offset_y + node.padding.top);

		compute_offset(
			child_node,
			Size::new(
				offset.width + local_offset_x + node.padding.left,
				offset.height + local_offset_y + node.padding.top,
			),
		);
	}
}

fn compute_flex_offset(node: &mut FlexLayoutNode, offset: Size) {
	let self_resolved = &mut node.resolved;

	self_resolved.update_x(offset.width);
	self_resolved.update_y(offset.height);

	if node.child_head.is_null() {
		return;
	}

	let inner_width = self_resolved.rect.width - node.padding.left - node.padding.right;
	let inner_height = self_resolved.rect.height - node.padding.top - node.padding.bottom;

	match node.direction {
		Direction::Row => {
			let mut pre_child_pointer = node.child_head;
			let mut child_pointer = node.child_head;
			let mut accum_height = match node.wrap_mode {
				WrapMode::Wrap | WrapMode::NoWrap => {
					offset.height + node.padding.top + node.cross_axis_gap_edge
				}
				WrapMode::WrapReverse => {
					offset.height + self_resolved.rect.height - node.padding.bottom
				}
			};

			loop {
				let mut occupied_width = 0f32;
				let mut occupied_height = 0f32;
				let mut line_count = 0;

				loop {
					line_count += 1;
					let pre_child_link = unsafe { &mut *pre_child_pointer };
					let pre_child = unsafe { &mut *pre_child_link.child };
					let pre_layout = pre_child.resolved();
					occupied_width += pre_layout.rect.width;
					occupied_height = occupied_height.max(pre_layout.rect.height);

					if pre_child_link.is_eol {
						break;
					}

					pre_child_pointer = pre_child_link.next;
				}

				let (gap_edge, gap_between) = resolve_flex_placement(
					node.row_placement,
					inner_width,
					occupied_width,
					line_count,
					node.row_gap,
				);
				let mut accum_width = offset.width + node.padding.left + gap_edge;

				loop {
					let child_link = unsafe { &mut *child_pointer };
					let child = unsafe { &mut *child_link.child };
					let layout = child.resolved_mut();

					layout.update_x(accum_width);

					let child_local_height_offset = match node.line_placement {
						Placement::Start => 0f32,
						Placement::End => occupied_height - layout.rect.height,
						Placement::Center => (occupied_height - layout.rect.height) / 2f32,
					};
					let child_height_offset = match node.wrap_mode {
						WrapMode::Wrap | WrapMode::NoWrap => {
							accum_height + child_local_height_offset
						}
						WrapMode::WrapReverse => {
							accum_height - occupied_height + child_local_height_offset
						}
					};

					layout.update_y(child_height_offset);

					if child_link.is_eol {
						break;
					}

					accum_width += gap_between + node.row_gap + layout.rect.width;

					child_pointer = child_link.next;
				}

				let pre_child_link = unsafe { &mut *pre_child_pointer };
				if pre_child_link.next.is_null() {
					break;
				}

				pre_child_pointer = pre_child_link.next;
				child_pointer = pre_child_link.next;

				match node.wrap_mode {
					WrapMode::Wrap | WrapMode::NoWrap => {
						accum_height +=
							node.cross_axis_gap_between + node.col_gap + occupied_height;
					}
					WrapMode::WrapReverse => {
						accum_height -=
							node.cross_axis_gap_between + node.col_gap + occupied_height;
					}
				}
			}
		}
		Direction::Column => {
			let mut pre_child_pointer = node.child_head;
			let mut child_pointer = node.child_head;
			let mut accum_width = match node.wrap_mode {
				WrapMode::Wrap | WrapMode::NoWrap => {
					offset.width + node.padding.left + node.cross_axis_gap_edge
				}
				WrapMode::WrapReverse => {
					offset.width + self_resolved.rect.width - node.padding.right
				}
			};

			loop {
				let mut occupied_width = 0f32;
				let mut occupied_height = 0f32;
				let mut line_count = 0;

				loop {
					line_count += 1;
					let pre_child_link = unsafe { &mut *pre_child_pointer };
					let pre_child = unsafe { &mut *pre_child_link.child };
					let pre_layout = pre_child.resolved();
					occupied_width = occupied_width.max(pre_layout.rect.width);
					occupied_height += pre_layout.rect.height;

					if pre_child_link.is_eol {
						break;
					}

					pre_child_pointer = pre_child_link.next;
				}

				let (gap_edge, gap_between) = resolve_flex_placement(
					node.col_placement,
					inner_height,
					occupied_height,
					line_count,
					node.col_gap,
				);
				let mut accum_height = offset.height + node.padding.top + gap_edge;

				loop {
					let child_link = unsafe { &mut *child_pointer };
					let child = unsafe { &mut *child_link.child };
					let layout = child.resolved_mut();

					layout.update_y(accum_height);

					let child_local_width_offset = match node.line_placement {
						Placement::Start => 0f32,
						Placement::End => occupied_width - layout.rect.width,
						Placement::Center => (occupied_width - layout.rect.width) / 2f32,
					};
					let child_width_offset = match node.wrap_mode {
						WrapMode::Wrap | WrapMode::NoWrap => accum_width + child_local_width_offset,
						WrapMode::WrapReverse => {
							accum_width - occupied_width + child_local_width_offset
						}
					};

					layout.update_x(child_width_offset);

					if child_link.is_eol {
						break;
					}

					accum_height += gap_between + node.col_gap + layout.rect.height;

					child_pointer = child_link.next;
				}

				let pre_child_link = unsafe { &mut *pre_child_pointer };
				if pre_child_link.next.is_null() {
					break;
				}

				pre_child_pointer = pre_child_link.next;
				child_pointer = pre_child_link.next;

				match node.wrap_mode {
					WrapMode::Wrap | WrapMode::NoWrap => {
						accum_width += node.cross_axis_gap_between + node.row_gap + occupied_width;
					}
					WrapMode::WrapReverse => {
						accum_width -= node.cross_axis_gap_between + node.row_gap + occupied_width;
					}
				}
			}
		}
	}
}

fn compute_grid_size(node: &mut GridLayoutNode, constraint: Constraint, preferred: Preferred) {
	let outer_width_constraint =
		resolve_constraint(preferred.width, node.min_width, node.max_width, constraint.max_width);
	let outer_height_constraint = resolve_constraint(
		preferred.height,
		node.min_height,
		node.max_height,
		constraint.max_height,
	);

	let inner_width_constraint = outer_width_constraint - node.padding.left - node.padding.right;
	let inner_height_constraint = outer_height_constraint - node.padding.top - node.padding.bottom;

	match node.direction {
		Direction::Row => {
			let mut row_track_sizes: Vec<f32> = vec![0f32; node.wrap_size];
			let mut col_track_sizes: Vec<f32> =
				vec![0f32; node.item_count.div_ceil(node.wrap_size)];

			let mut child_pointer = node.child_head;
			let mut idx = 0;
			loop {
				let child_link = unsafe { &*child_pointer };
				{
					let child = unsafe { &mut *child_link.child };
					compute_size(
						child,
						Constraint::new(inner_width_constraint, inner_height_constraint),
					);
				}
				let child = unsafe { &mut *child_link.child };
				row_track_sizes[idx % node.wrap_size] =
					row_track_sizes[idx % node.wrap_size].max(child.resolved().rect.width);
				col_track_sizes[idx / node.wrap_size] =
					col_track_sizes[idx / node.wrap_size].max(child.resolved().rect.height);
				if child_link.next.is_null() {
					break;
				}
				child_pointer = child_link.next;
				idx += 1;
			}

			node.row_axis_sizes = row_track_sizes;
			node.col_axis_sizes = col_track_sizes;
		}
		Direction::Column => {
			let mut col_track_sizes: Vec<f32> = vec![0f32; node.wrap_size];
			let mut row_track_sizes: Vec<f32> =
				vec![0f32; node.item_count.div_ceil(node.wrap_size)];

			let mut child_pointer = node.child_head;
			let mut idx = 0;
			loop {
				let child_link = unsafe { &*child_pointer };
				{
					let child = unsafe { &mut *child_link.child };
					compute_size(
						child,
						Constraint::new(inner_width_constraint, inner_height_constraint),
					);
				}
				let child = unsafe { &mut *child_link.child };
				col_track_sizes[idx % node.wrap_size] =
					col_track_sizes[idx % node.wrap_size].max(child.resolved().rect.width);
				row_track_sizes[idx / node.wrap_size] =
					row_track_sizes[idx / node.wrap_size].max(child.resolved().rect.height);
				if child_link.next.is_null() {
					break;
				}
				child_pointer = child_link.next;
				idx += 1;
			}

			node.row_axis_sizes = row_track_sizes;
			node.col_axis_sizes = col_track_sizes;
		}
	}
}

fn compute_grid_offset(node: &mut GridLayoutNode, offset: Size) {
	let self_resolved = &mut node.resolved;

	self_resolved.update_x(offset.width);
	self_resolved.update_y(offset.height);

	if node.child_head.is_null() {
		return;
	}

	let inner_width = self_resolved.rect.width - node.padding.left - node.padding.right;
	let inner_height = self_resolved.rect.height - node.padding.top - node.padding.bottom;

	let (occupied_row_gap, occupied_col_gap) = match node.direction {
		Direction::Row => (
			node.row_gap * node.wrap_size as f32,
			node.col_gap * node.item_count.div_ceil(node.wrap_size) as f32,
		),
		Direction::Column => (
			node.row_gap * node.item_count.div_ceil(node.wrap_size) as f32,
			node.col_gap * node.wrap_size as f32,
		),
	};

	// TODO: SIMDによるsumの高速化
	let occupied_col_placement =
		inner_height - node.col_axis_sizes.iter().sum::<f32>() - occupied_col_gap;
	let (occupied_row_placement_edge, occupied_row_placement_between) = resolve_flex_placement(
		node.row_placement,
		inner_width,
		node.row_axis_sizes.iter().sum::<f32>(),
		match node.direction {
			Direction::Row => node.wrap_size,
			Direction::Column => node.item_count.div_ceil(node.wrap_size),
		},
		node.row_gap,
	);
	let (occupied_col_placement_edge, occupied_col_placement_between) = resolve_flex_placement(
		node.col_placement,
		inner_height,
		node.col_axis_sizes.iter().sum::<f32>(),
		match node.direction {
			Direction::Row => node.item_count.div_ceil(node.wrap_size),
			Direction::Column => node.wrap_size,
		},
		node.col_gap,
	);

	let mut accum_width = occupied_row_placement_edge;
	let mut accum_height = occupied_col_placement_edge;
	let mut child_pointer = node.child_head;
	let mut axis_idx = 0;

	println!(
		"{} {}, {} {}",
		occupied_row_placement_edge,
		occupied_row_placement_between,
		occupied_col_placement_edge,
		occupied_col_placement_between
	);

	match node.direction {
		Direction::Row => 'col: loop {
			for i in 0..node.wrap_size {
				let child_link = unsafe { &*child_pointer };
				let child = unsafe { &mut *child_link.child };

				let child_local_width_offset = match child_link.row_placement {
					Placement::Start => 0f32,
					Placement::End => node.row_axis_sizes[i] - child.resolved().rect.width,
					Placement::Center => {
						(node.row_axis_sizes[i] - child.resolved().rect.width) / 2f32
					}
				};
				let child_local_height_offset = match child_link.col_placement {
					Placement::Start => 0f32,
					Placement::End => node.col_axis_sizes[axis_idx] - child.resolved().rect.height,
					Placement::Center => {
						(node.col_axis_sizes[axis_idx] - child.resolved().rect.height) / 2f32
					}
				};

				compute_offset(
					child,
					Size::new(
						accum_width + child_local_width_offset,
						accum_height + child_local_height_offset,
					),
				);

				if child_link.next.is_null() {
					break 'col;
				}

				child_pointer = child_link.next;

				accum_width +=
					occupied_row_placement_between + node.row_gap + node.row_axis_sizes[i];
			}
			accum_width = occupied_row_placement_edge;
			accum_height +=
				occupied_col_placement_between + node.col_gap + node.col_axis_sizes[axis_idx];
			axis_idx += 1;
		},
		Direction::Column => {}
	}
}

fn resolve_flex_placement(
	placement: FlexPlacement,
	available_size: f32,
	content_size: f32,
	item_count: usize,
	gap: f32,
) -> (f32, f32) {
	match placement {
		FlexPlacement::Start => (0f32, 0f32),
		FlexPlacement::End => {
			let min_gap_sum = gap * (item_count - 1) as f32;
			(available_size - content_size - min_gap_sum, 0f32)
		}
		FlexPlacement::Center => {
			let min_gap_sum = gap * (item_count - 1) as f32;
			((available_size - content_size - min_gap_sum) / 2f32, 0f32)
		}
		FlexPlacement::SafeCenter => {
			if available_size > content_size {
				let min_gap_sum = gap * (item_count - 1) as f32;
				((available_size - content_size - min_gap_sum) / 2f32, 0f32)
			} else {
				(0f32, 0f32)
			}
		}
		FlexPlacement::SpaceBetween => {
			if item_count == 1 {
				return (0f32, 0f32);
			}
			let min_gap_sum = gap * (item_count - 1) as f32;
			let placement_size = available_size - content_size - min_gap_sum;
			(placement_size / (item_count - 1) as f32, 0f32)
		}
		FlexPlacement::SpaceAround => {
			let min_gap_sum = gap * (item_count - 1) as f32;
			let placement_size = available_size - content_size - min_gap_sum;
			let placement_edge = placement_size / (item_count as f32 * 2f32);
			(placement_edge, placement_edge * 2f32)
		}
		FlexPlacement::SpaceEvenly => {
			let min_gap_sum = gap * (item_count - 1) as f32;
			let placement_size = available_size - content_size - min_gap_sum;
			let placement_gap = placement_size / (item_count + 1) as f32;
			(placement_gap, placement_gap)
		}
	}
}
