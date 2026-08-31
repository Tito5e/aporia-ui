use crate::{
	geometry::{Constraint, DimensionResolver},
	node::{BoxLayoutNode, FlexLayoutNode, LayoutNode, RatioLayoutNode},
	style::{Direction, RatioMode, WrapMode},
};

pub(crate) fn compute_size(node: &mut LayoutNode, constraint: Constraint) {
	// Reset Dirty Flags
	node.clear_dirty();

	match node {
		LayoutNode::Box(node) => {
			compute_box_size(node, constraint);
		}
		LayoutNode::Ratio(node) => {
			compute_ratio_size(node, constraint);
		}
		LayoutNode::Flex(node) => {
			compute_flex_size(node, constraint);
		}
		LayoutNode::Grid(node) => {
			compute_grid_size(node, constraint);
		}
	}
}

fn compute_box_size(node: &mut BoxLayoutNode, constraint: Constraint) {
	let outer_width_constraint = node.resolved.width;
	let outer_height_constraint = node.resolved.height;

	if !node.child.ptr.is_null() {
		let inner_width_constraint =
			outer_width_constraint - node.padding.left - node.padding.right;
		let inner_height_constraint =
			outer_height_constraint - node.padding.top - node.padding.bottom;

		let child_node = unsafe { &mut *node.child.ptr };

		child_node.set_size(
			child_node.width().to_constraint(
				child_node.min_width(),
				child_node.max_width(),
				inner_width_constraint,
			),
			child_node.height().to_constraint(
				child_node.min_height(),
				child_node.max_height(),
				inner_height_constraint,
			),
		);

		compute_size(child_node, Constraint::new(inner_width_constraint, inner_height_constraint));
	}

	let self_resolved = &mut node.resolved;

	if node.width.is_none() {
		let mut self_width = if !node.child.ptr.is_null() {
			let child = unsafe { &mut *node.child.ptr };
			let child_resolved = child.resolved();

			child_resolved.width + node.padding.left + node.padding.right
		} else {
			node.padding.left + node.padding.right
		};

		if let Some(max_width) = node.max_width {
			self_width = max_width.resolve(constraint.max_width).min(self_width);
		}
		if let Some(min_width) = node.min_width {
			self_width = min_width.resolve(constraint.max_width).max(self_width);
		}

		self_resolved.update_width(self_width);
	}
	if node.height.is_none() {
		let mut self_height = if !node.child.ptr.is_null() {
			let child = unsafe { &mut *node.child.ptr };
			let child_resolved = child.resolved();

			child_resolved.height + node.padding.top + node.padding.bottom
		} else {
			node.padding.top + node.padding.bottom
		};

		if let Some(max_height) = node.max_height {
			self_height = max_height.resolve(constraint.max_height).min(self_height);
		}
		if let Some(min_height) = node.min_height {
			self_height = min_height.resolve(constraint.max_height).max(self_height);
		}

		self_resolved.update_height(self_height);
	}
}

fn compute_ratio_size(node: &mut RatioLayoutNode, constraint: Constraint) {
	let outer_width_constraint = node.resolved.width;
	let outer_height_constraint = node.resolved.height;

	let (outer_width_constraint, outer_height_constraint) = match node.mode {
		RatioMode::Width => {
			let w = outer_width_constraint;
			let h = outer_width_constraint / node.ratio;

			(w, h)
		}
		RatioMode::Height => {
			let w = outer_height_constraint * node.ratio;
			let h = outer_height_constraint;

			(w, h)
		}
		RatioMode::Fit => {
			let constraint_ratio = outer_width_constraint / outer_height_constraint;

			let (w, h) = if constraint_ratio > node.ratio {
				let w = outer_height_constraint * node.ratio;
				let h = outer_height_constraint;

				(w, h)
			} else {
				let w = outer_width_constraint;
				let h = outer_width_constraint / node.ratio;

				(w, h)
			};

			(w, h)
		}
		RatioMode::Fill => {
			let constraint_ratio = outer_width_constraint / outer_height_constraint;

			let (w, h) = if constraint_ratio < node.ratio {
				let w = outer_height_constraint * node.ratio;
				let h = outer_height_constraint;

				(w, h)
			} else {
				let w = outer_width_constraint;
				let h = outer_width_constraint / node.ratio;

				(w, h)
			};

			(w, h)
		}
	};

	let self_resolved = &mut node.resolved;

	self_resolved.update_width(outer_width_constraint);
	self_resolved.update_height(outer_height_constraint);

	let inner_width_constraint = outer_width_constraint - node.padding.left - node.padding.right;
	let inner_height_constraint = outer_height_constraint - node.padding.top - node.padding.bottom;

	if !node.child.ptr.is_null() {
		let child_node = unsafe { &mut *node.child.ptr };

		child_node.set_size(
			child_node.width().to_constraint(
				child_node.min_width(),
				child_node.max_width(),
				inner_width_constraint,
			),
			child_node.height().to_constraint(
				child_node.min_height(),
				child_node.max_height(),
				inner_height_constraint,
			),
		);

		compute_size(child_node, Constraint::new(inner_width_constraint, inner_height_constraint));
	}
}

fn compute_flex_size(node: &mut FlexLayoutNode, constraint: Constraint) {
	let outer_width_constraint = node.resolved.width;
	let outer_height_constraint = node.resolved.height;

	let inner_width_constraint = outer_width_constraint - node.padding.left - node.padding.right;
	let inner_height_constraint = outer_height_constraint - node.padding.top - node.padding.bottom;

	let mut occupied_width = 0f32;
	let mut occupied_height = 0f32;

	if !node.children.is_empty() {
		match node.direction {
			Direction::Row => match node.wrap_mode {
				WrapMode::NoWrap => {
					for child_ptr in &mut node.children {
						child_ptr.is_split = false;
						let child_node = unsafe { &mut *child_ptr.ptr };

						child_node.set_size(
							child_node.width().to_constraint(
								child_node.min_width(),
								child_node.max_width(),
								inner_width_constraint,
							),
							child_node.height().to_constraint(
								child_node.min_height(),
								child_node.max_height(),
								inner_height_constraint,
							),
						);

						compute_size(
							child_node,
							Constraint::new(inner_width_constraint, inner_height_constraint),
						);
						let child_resolved = child_node.resolved();
						occupied_width += child_resolved.width;
						occupied_height = occupied_height.max(child_resolved.height);
					}
					occupied_width += node.row_gap * (node.children.len() - 1) as f32;
				}
				WrapMode::Wrap | WrapMode::WrapReverse => {
					let mut line_occupied_width = 0f32;
					let mut line_occupied_height = 0f32;
					let mut cross_axis_line_count = 1;

					for child_ptr in &node.children {
						let child_node = unsafe { &mut *child_ptr.ptr };

						child_node.set_size(
							child_node.width().to_constraint(
								child_node.min_width(),
								child_node.max_width(),
								inner_width_constraint,
							),
							child_node.height().to_constraint(
								child_node.min_height(),
								child_node.max_height(),
								inner_height_constraint,
							),
						);

						compute_size(
							child_node,
							Constraint::new(inner_width_constraint, inner_height_constraint),
						);

						let child_resolved = child_node.resolved();

						if inner_width_constraint < line_occupied_width + child_resolved.width {
							occupied_width = occupied_width.max(line_occupied_width);
							occupied_height += line_occupied_height;

							if !child.next.is_null() {
								occupied_height += node.col_gap;
								child_pointer = child.next;

								line_occupied_width = child_resolved.rect.width;
								line_occupied_height = child_resolved.rect.height;
								continue;
							} else {
								occupied_width = occupied_width.max(child_resolved.rect.width);
								occupied_height += child_resolved.rect.height;

								break;
							}
						}

						line_occupied_width += child_resolved.rect.width;
						line_occupied_height = line_occupied_height.max(child_resolved.rect.height);

						if child.next.is_null() {
							cross_axis_line_count += 1;
							child.is_eol = true;

							occupied_width = occupied_width.max(line_occupied_width);
							occupied_height += line_occupied_height;

							break;
						} else {
							child.is_eol = false;
							child_pointer = child.next;
							line_occupied_width += node.row_gap;
						}
					}
				}
			},
			Direction::Column => match node.wrap_mode {
				WrapMode::NoWrap => {
					let mut child_pointer = node.child_head;

					loop {
						let child = unsafe { &mut *child_pointer };
						{
							let child_node = unsafe { &mut *child.child };
							compute_size(
								child_node,
								Constraint::new(inner_width_constraint, inner_height_constraint),
								Preferred::new(child.width, child.height),
							);
						}
						let child_node = unsafe { &mut *child.child };
						let child_resolved = child_node.resolved();
						occupied_width = occupied_width.max(child_resolved.rect.width);
						occupied_height += child_resolved.rect.height;

						if child.next.is_null() {
							child.is_eol = true;
							break;
						} else {
							child.is_eol = false;
							child_pointer = child.next;
							occupied_height += node.col_gap;
						}
					}
				}
				WrapMode::Wrap | WrapMode::WrapReverse => {
					let mut line_occupied_width = 0f32;
					let mut line_occupied_height = 0f32;

					let mut child_pointer = node.child_head;

					loop {
						let child = unsafe { &mut *child_pointer };
						{
							let child_node = unsafe { &mut *child.child };
							compute_size(
								child_node,
								Constraint::new(inner_width_constraint, inner_height_constraint),
								Preferred::new(child.width, child.height),
							);
						}

						let child_node = unsafe { &mut *child.child };
						let child_resolved = child_node.resolved();

						if inner_height_constraint
							< line_occupied_height + child_resolved.rect.height
						{
							let previous_child = unsafe { &mut *child.prev };
							previous_child.is_eol = true;

							occupied_width += line_occupied_width;
							occupied_height = occupied_height.max(line_occupied_height);

							if !child.next.is_null() {
								occupied_width += node.row_gap;
								child_pointer = child.next;

								line_occupied_width = child_resolved.rect.width;
								line_occupied_height = child_resolved.rect.height;
								continue;
							} else {
								occupied_width += child_resolved.rect.width;
								occupied_height = occupied_height.max(child_resolved.rect.height);

								break;
							}
						}

						line_occupied_width = line_occupied_width.max(child_resolved.rect.width);
						line_occupied_height += child_resolved.rect.height;

						if child.next.is_null() {
							child.is_eol = true;

							occupied_width += line_occupied_width;
							occupied_height = occupied_height.max(line_occupied_height);

							break;
						} else {
							child.is_eol = false;
							child_pointer = child.next;
							line_occupied_height += node.col_gap;
						}
					}
				}
			},
		}
	}

	let self_resolved = &mut node.resolved;

	let outer_width = match preferred.width {
		Some(_) => {
			self_resolved.update_width(outer_width_constraint);
			outer_width_constraint
		}
		None => {
			let mut self_width = occupied_width + node.padding.left + node.padding.right;

			if let Some(max_width) = node.max_width {
				self_width = max_width.resolve(constraint.max_width).min(self_width);
			}
			if let Some(min_width) = node.min_width {
				self_width = min_width.resolve(constraint.max_width).max(self_width);
			}
			self_width = constraint.max_width.min(self_width);

			self_resolved.update_width(self_width);
			self_width
		}
	};
	let outer_height = match preferred.height {
		Some(_) => {
			self_resolved.update_height(outer_height_constraint);
			outer_height_constraint
		}
		None => {
			let mut self_height = occupied_height + node.padding.top + node.padding.bottom;

			if let Some(max_height) = node.max_height {
				self_height = max_height.resolve(constraint.max_height).min(self_height);
			}
			if let Some(min_height) = node.min_height {
				self_height = min_height.resolve(constraint.max_height).max(self_height);
			}
			self_height = constraint.max_height.min(self_height);

			self_resolved.update_height(self_height);
			self_height
		}
	};

	if !node.child_head.is_null() {
		let inner_width = outer_width - node.padding.left - node.padding.right;
		let inner_height = outer_height - node.padding.top - node.padding.bottom;

		match node.direction {
			Direction::Column => {
				let (gap_edge, gap_between) = resolve_flex_placement(
					node.row_placement,
					inner_width,
					occupied_width,
					cross_axis_line_count,
					node.row_gap,
				);
				node.cross_axis_gap_edge = gap_edge;
				node.cross_axis_gap_between = gap_between;
			}
			Direction::Row => {
				let (gap_edge, gap_between) = resolve_flex_placement(
					node.col_placement,
					inner_height,
					occupied_height,
					cross_axis_line_count,
					node.col_gap,
				);
				node.cross_axis_gap_edge = gap_edge;
				node.cross_axis_gap_between = gap_between;
			}
		}
	}
}
