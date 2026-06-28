use crate::geometry::{Constraint, Dimension};
use crate::node::{BoxLayoutNode, FlexLayoutNode, LayoutNode, LayoutNodeExt, RatioLayoutNode};
use crate::style::{Direction, FlexPlacement, Placement, RatioMode, WrapMode};
use aporia_core::geometry::Size;

pub fn compute_layout(node: &mut LayoutNode, constraint: Constraint) {
    // PASS-1
    compute_size(node, constraint);

    // PASS-2
    compute_offset(node, Size::new(0f32, 0f32));
}

fn compute_size(node: &mut LayoutNode, constraint: Constraint) {
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
    }
}

fn compute_offset(node: &mut LayoutNode, offset: Size) {
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
    }
}

fn compute_box_size(node: &mut BoxLayoutNode, constraint: Constraint) {
    let outer_width_constraint =
        resolve_constraint(node.width, node.min_width, node.max_width, constraint.max_width);
    let outer_height_constraint =
        resolve_constraint(node.height, node.min_height, node.max_height, constraint.max_height);

    if !node.child.is_null() {
        let inner_width_constraint =
            outer_width_constraint - node.padding.left - node.padding.right;
        let inner_height_constraint =
            outer_height_constraint - node.padding.top - node.padding.bottom;

        compute_size(
            unsafe { &mut *node.child },
            Constraint::new(inner_width_constraint, inner_height_constraint),
        );
    }

    let self_resolved = unsafe { &mut *node.resolved };

    match node.width {
        Some(_) => {
            self_resolved.update_width(outer_width_constraint);
        }
        None => {
            let mut self_width = if !node.child.is_null() {
                let child = unsafe { &*node.child };
                let child_resolved = child.resolved_mut();

                child_resolved.rect.width + node.padding.left + node.padding.right
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
    }
    match node.height {
        Some(_) => {
            self_resolved.update_height(outer_height_constraint);
        }
        None => {
            let mut self_height = if !node.child.is_null() {
                let child = unsafe { &*node.child };
                let child_resolved = child.resolved_mut();

                child_resolved.rect.height + node.padding.top + node.padding.bottom
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
}

fn compute_box_offset(node: &mut BoxLayoutNode, offset: Size) {
    let self_resolved = unsafe { &mut *node.resolved };

    self_resolved.update_x(offset.width);
    self_resolved.update_y(offset.height);

    if !node.child.is_null() {
        let child = unsafe { &mut *node.child };
        let inner_resolved = child.resolved_mut();

        let local_offset_x = match node.row_placement {
            Placement::Start => 0f32,
            Placement::End => {
                let width_margin = self_resolved.rect.width
                    - inner_resolved.rect.width
                    - node.padding.left
                    - node.padding.right;

                width_margin
            }
            Placement::Center => {
                let width_margin = self_resolved.rect.width
                    - inner_resolved.rect.width
                    - node.padding.left
                    - node.padding.right;

                width_margin / 2f32
            }
        };

        let local_offset_y = match node.col_placement {
            Placement::Start => 0f32,
            Placement::End => {
                let height_margin = self_resolved.rect.height
                    - inner_resolved.rect.height
                    - node.padding.top
                    - node.padding.bottom;

                height_margin
            }
            Placement::Center => {
                let height_margin = self_resolved.rect.height
                    - inner_resolved.rect.height
                    - node.padding.top
                    - node.padding.bottom;

                height_margin / 2f32
            }
        };

        compute_offset(
            child,
            Size::new(
                offset.width + local_offset_x + node.padding.left,
                offset.height + local_offset_y + node.padding.top,
            ),
        );
    }
}

fn compute_ratio_size(node: &mut RatioLayoutNode, constraint: Constraint) {
    let mut constraint_width = constraint.max_width;
    let mut constraint_height = constraint.max_height;

    if let Some(max_width) = node.max_width {
        constraint_width = constraint_width.min(max_width.resolve(constraint.max_width));
    }
    if let Some(max_height) = node.max_height {
        constraint_height = constraint_height.min(max_height.resolve(constraint.max_height));
    }

    let (width, height) = match node.mode {
        RatioMode::Width => {
            let w = constraint_width;
            let h = constraint_width / node.ratio;

            (w, h)
        }
        RatioMode::Height => {
            let w = constraint_height * node.ratio;
            let h = constraint_height;

            (w, h)
        }
        RatioMode::Fit => {
            let constraint_ratio = constraint_width / constraint_height;

            let (w, h) = if constraint_ratio > node.ratio {
                let w = constraint_height * node.ratio;
                let h = constraint_height;

                (w, h)
            } else {
                let w = constraint_width;
                let h = constraint_width / node.ratio;

                (w, h)
            };

            (w, h)
        }
        RatioMode::Fill => {
            let constraint_ratio = constraint_width / constraint_height;

            let (w, h) = if constraint_ratio < node.ratio {
                let w = constraint_height * node.ratio;
                let h = constraint_height;

                (w, h)
            } else {
                let w = constraint_width;
                let h = constraint_width / node.ratio;

                (w, h)
            };

            (w, h)
        }
    };

    let self_resolved = unsafe { &mut *node.resolved };

    self_resolved.update_width(width);
    self_resolved.update_height(height);

    let inner_w = width - node.padding.left - node.padding.right;
    let inner_h = height - node.padding.top - node.padding.bottom;

    if !node.child.is_null() {
        compute_size(unsafe { &mut *node.child }, Constraint::new(inner_w, inner_h));
    }
}

fn compute_ratio_offset(node: &mut RatioLayoutNode, offset: Size) {
    let self_resolved = unsafe { &mut *node.resolved };

    self_resolved.update_x(offset.width);
    self_resolved.update_y(offset.height);

    if !node.child.is_null() {
        let child = unsafe { &mut *node.child };
        let inner_resolved = child.resolved_mut();

        let local_offset_x = match node.row_placement {
            Placement::Start => 0f32,
            Placement::End => {
                let width_margin = self_resolved.rect.width - inner_resolved.rect.width;

                width_margin
            }
            Placement::Center => {
                let width_margin = self_resolved.rect.width - inner_resolved.rect.width;

                width_margin / 2f32
            }
        };

        let local_offset_y = match node.col_placement {
            Placement::Start => 0f32,
            Placement::End => {
                let height_margin = self_resolved.rect.height - inner_resolved.rect.height;

                height_margin
            }
            Placement::Center => {
                let height_margin = self_resolved.rect.height - inner_resolved.rect.height;

                height_margin / 2f32
            }
        };

        compute_offset(
            child,
            Size::new(
                offset.width + local_offset_x + node.padding.left,
                offset.height + local_offset_y + node.padding.top,
            ),
        );
    }
}

fn compute_flex_size(node: &mut FlexLayoutNode, constraint: Constraint) {
    let outer_width_constraint =
        resolve_constraint(node.width, node.min_width, node.max_width, constraint.max_width);
    let outer_height_constraint =
        resolve_constraint(node.height, node.min_height, node.max_height, constraint.max_height);

    let inner_width_constraint = outer_width_constraint - node.padding.left - node.padding.right;
    let inner_height_constraint = outer_height_constraint - node.padding.top - node.padding.bottom;

    let mut occupied_width = 0f32;
    let mut occupied_height = 0f32;
    let mut cross_axis_line_count = 0;

    if !node.child_head.is_null() {
        match node.direction {
            Direction::Row => match node.wrap_mode {
                WrapMode::NoWrap => {
                    let mut child_pointer = node.child_head;
                    cross_axis_line_count = 1;

                    loop {
                        let child = unsafe { &mut *child_pointer };
                        {
                            let child_node = unsafe { &mut *child.child };
                            compute_size(
                                child_node,
                                Constraint::new(inner_width_constraint, inner_height_constraint),
                            );
                        }
                        let child_node = unsafe { &mut *child.child };
                        let child_resolved = child_node.resolved_mut();
                        occupied_width += child_resolved.rect.width;
                        occupied_height = occupied_height.max(child_resolved.rect.height);

                        if child.next.is_null() {
                            child.is_eol = true;
                            break;
                        } else {
                            child.is_eol = false;
                            child_pointer = child.next;
                            occupied_width += node.row_gap;
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
                            );
                        }

                        let child_node = unsafe { &mut *child.child };
                        let child_resolved = child_node.resolved_mut();

                        if inner_width_constraint < line_occupied_width + child_resolved.rect.width
                        {
                            let previous_child = unsafe { &mut *child.prev };
                            cross_axis_line_count += 1;
                            previous_child.is_eol = true;

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
                            );
                        }
                        let child_node = unsafe { &mut *child.child };
                        let child_resolved = child_node.resolved_mut();
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
                            );
                        }

                        let child_node = unsafe { &mut *child.child };
                        let child_resolved = child_node.resolved_mut();

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

    let self_resolved = unsafe { &mut *node.resolved };

    let outer_width = match node.width {
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
    let outer_height = match node.height {
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

fn compute_flex_offset(node: &mut FlexLayoutNode, offset: Size) {
    let self_resolved = unsafe { &mut *node.resolved };

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
                    let pre_child = unsafe { &*pre_child_link.child };
                    let pre_layout = pre_child.resolved_mut();
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
                    let child = unsafe { &*child_link.child };
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
                    let pre_child = unsafe { &*pre_child_link.child };
                    let pre_layout = pre_child.resolved_mut();
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
                    let child = unsafe { &*child_link.child };
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

fn resolve_constraint(
    preferred: Option<Dimension>,
    min: Option<Dimension>,
    max: Option<Dimension>,
    constraint: f32,
) -> f32 {
    let mut result;

    if let Some(preferred) = preferred {
        result = preferred.resolve(constraint);

        if let Some(max) = max {
            result = max.resolve(constraint).min(result);
        }
        if let Some(min) = min {
            result = min.resolve(constraint).max(result);
        }
    } else {
        if let Some(max) = max {
            result = max.resolve(constraint);
        } else {
            result = constraint;
        }
    }

    result
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
