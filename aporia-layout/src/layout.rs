use crate::geometry::Constraint;
use crate::node::{BoxLayoutNode, FlexLayoutNode, LayoutNode, LayoutNodeExt, RatioLayoutNode};
use crate::style::{Direction, Placement, RatioMode, WrapMode};
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
    let mut outer_width_constraint;
    let mut outer_height_constraint;

    if let Some(preferred_width) = node.width {
        outer_width_constraint = preferred_width.resolve(constraint.max_width);

        if let Some(max_width) = node.max_width {
            outer_width_constraint =
                max_width.resolve(constraint.max_width).min(outer_width_constraint);
        }
        if let Some(min_width) = node.min_width {
            outer_width_constraint =
                min_width.resolve(constraint.max_width).max(outer_width_constraint);
        }
    } else {
        outer_width_constraint = constraint.max_width;
    }
    if let Some(preferred_height) = node.height {
        outer_height_constraint = preferred_height.resolve(constraint.max_height);

        if let Some(max_height) = node.max_height {
            outer_height_constraint =
                max_height.resolve(constraint.max_height).min(outer_height_constraint);
        }
        if let Some(min_height) = node.min_height {
            outer_height_constraint =
                min_height.resolve(constraint.max_height).max(outer_height_constraint);
        }
    } else {
        outer_height_constraint = constraint.max_height;
    }

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
            if (self_resolved.rect.width - outer_width_constraint).abs() > 0.01 {
                self_resolved.is_width_changed = true;
                self_resolved.rect.width = outer_width_constraint;
            }
        }
        None => {
            let mut self_width = if node.child.is_null() {
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

            self_width = constraint.max_width.min(self_width);

            if (self_resolved.rect.width - self_width).abs() > 0.01 {
                self_resolved.is_width_changed = true;
                self_resolved.rect.width = self_width;
            }
        }
    }
    match node.height {
        Some(_) => {
            if (self_resolved.rect.height - outer_height_constraint).abs() > 0.01 {
                self_resolved.is_height_changed = true;
                self_resolved.rect.height = outer_height_constraint;
            }
        }
        None => {
            let mut self_height = if node.child.is_null() {
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

            self_height = constraint.max_height.min(self_height);

            if (self_resolved.rect.height - self_height).abs() > 0.01 {
                self_resolved.is_height_changed = true;
                self_resolved.rect.height = self_height;
            }
        }
    }
}

fn compute_box_offset(node: &mut BoxLayoutNode, offset: Size) {
    let self_resolved = unsafe { &mut *node.resolved };

    if (self_resolved.rect.x - offset.width).abs() > 0.01 {
        self_resolved.is_x_changed = true;
        self_resolved.rect.x = offset.width;
    }
    if (self_resolved.rect.y - offset.height).abs() > 0.01 {
        self_resolved.is_y_changed = true;
        self_resolved.rect.y = offset.height;
    }

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
    match node.mode {
        RatioMode::Width => {
            let max_w = node
                .max_width
                .map(|w| w.resolve(constraint.max_width))
                .unwrap_or(0.0)
                .min(constraint.max_width);

            let w = max_w;
            let h = max_w / node.ratio;

            let inner_w = w - node.padding.left - node.padding.right;
            let inner_h = h - node.padding.top - node.padding.bottom;

            if !node.child.is_null() {
                compute_size(unsafe { &mut *node.child }, Constraint::new(inner_w, inner_h));
            }
        }
        RatioMode::Height => {
            let max_h = node
                .max_height
                .map(|h| h.resolve(constraint.max_height))
                .unwrap_or(0.0)
                .min(constraint.max_height);

            let w = max_h;
            let h = max_h * node.ratio;

            let inner_w = w - node.padding.left - node.padding.right;
            let inner_h = h - node.padding.top - node.padding.bottom;

            if !node.child.is_null() {
                compute_size(unsafe { &mut *node.child }, Constraint::new(inner_w, inner_h));
            }
        }
        RatioMode::Fit => {
            let max_w = node
                .max_width
                .map(|w| w.resolve(constraint.max_width))
                .unwrap_or(0.0)
                .min(constraint.max_width);
            let max_h = node
                .max_height
                .map(|h| h.resolve(constraint.max_height))
                .unwrap_or(0.0)
                .min(constraint.max_height);

            let constraint_ratio = max_w / max_h;

            let (w, h) = if constraint_ratio > node.ratio {
                let w = max_h;
                let h = max_h * node.ratio;

                (w, h)
            } else {
                let w = max_w;
                let h = max_w / node.ratio;

                (w, h)
            };

            let inner_w = w - node.padding.left - node.padding.right;
            let inner_h = h - node.padding.top - node.padding.bottom;

            if !node.child.is_null() {
                compute_size(unsafe { &mut *node.child }, Constraint::new(inner_w, inner_h));
            }
        }
        RatioMode::Fill => {
            let max_w = node
                .max_width
                .map(|w| w.resolve(constraint.max_width))
                .unwrap_or(0.0)
                .min(constraint.max_width);
            let max_h = node
                .max_height
                .map(|h| h.resolve(constraint.max_height))
                .unwrap_or(0.0)
                .min(constraint.max_height);

            let constraint_ratio = max_w / max_h;

            let (w, h) = if constraint_ratio < node.ratio {
                let w = max_h;
                let h = max_h * node.ratio;

                (w, h)
            } else {
                let w = max_w;
                let h = max_w / node.ratio;

                (w, h)
            };

            let inner_w = w - node.padding.left - node.padding.right;
            let inner_h = h - node.padding.top - node.padding.bottom;

            if !node.child.is_null() {
                compute_size(unsafe { &mut *node.child }, Constraint::new(inner_w, inner_h));
            }
        }
    }
}

fn compute_ratio_offset(node: &mut RatioLayoutNode, offset: Size) {
    let self_resolved = unsafe { &mut *node.resolved };

    if (self_resolved.rect.x - offset.width).abs() > 0.01 {
        self_resolved.is_x_changed = true;
        self_resolved.rect.x = offset.width;
    }
    if (self_resolved.rect.y - offset.height).abs() > 0.01 {
        self_resolved.is_y_changed = true;
        self_resolved.rect.y = offset.height;
    }

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
    let mut outer_width_constraint;
    let mut outer_height_constraint;

    if let Some(preferred_width) = node.width {
        outer_width_constraint = preferred_width.resolve(constraint.max_width);

        if let Some(max_width) = node.max_width {
            outer_width_constraint =
                max_width.resolve(constraint.max_width).min(outer_width_constraint);
        }
        if let Some(min_width) = node.min_width {
            outer_width_constraint =
                min_width.resolve(constraint.max_width).max(outer_width_constraint);
        }
    } else {
        outer_width_constraint = constraint.max_width;
    }
    if let Some(preferred_height) = node.height {
        outer_height_constraint = preferred_height.resolve(constraint.max_height);

        if let Some(max_height) = node.max_height {
            outer_height_constraint =
                max_height.resolve(constraint.max_height).min(outer_height_constraint);
        }
        if let Some(min_height) = node.min_height {
            outer_height_constraint =
                min_height.resolve(constraint.max_height).max(outer_height_constraint);
        }
    } else {
        outer_height_constraint = constraint.max_height;
    }

    let inner_width_constraint = outer_width_constraint - node.padding.left - node.padding.right;
    let inner_height_constraint = outer_height_constraint - node.padding.top - node.padding.bottom;

    let mut preserve_width = 0f32;
    let mut preserve_height = 0f32;

    match node.direction {
        Direction::Row => match node.wrap_mode {
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
                    preserve_width += child_resolved.rect.width;
                    preserve_height = preserve_height.max(child_resolved.rect.height);

                    if child.next.is_null() {
                        child.is_eol = true;
                        break;
                    } else {
                        child.is_eol = false;
                        child_pointer = child.next;
                        preserve_width += node.row_gap;
                    }
                }
            }
            WrapMode::Wrap | WrapMode::WrapReverse => {
                let mut line_preserve_width = 0f32;
                let mut line_preserve_height = 0f32;

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

                    if inner_width_constraint < line_preserve_width + child_resolved.rect.width {
                        let previous_child = unsafe { &mut *child.prev };
                        previous_child.is_eol = true;

                        preserve_width = preserve_width.max(line_preserve_width);
                        preserve_height += line_preserve_height;

                        if !child.next.is_null() {
                            preserve_height += node.col_gap;
                            child_pointer = child.next;

                            line_preserve_width = child_resolved.rect.width;
                            line_preserve_height = child_resolved.rect.height;
                            continue;
                        } else {
                            preserve_width = preserve_width.max(child_resolved.rect.width);
                            preserve_height += child_resolved.rect.height;

                            break;
                        }
                    }

                    line_preserve_width += child_resolved.rect.width;
                    line_preserve_height = line_preserve_height.max(child_resolved.rect.height);

                    if child.next.is_null() {
                        child.is_eol = true;

                        preserve_width = preserve_width.max(line_preserve_width);
                        preserve_height += line_preserve_height;

                        break;
                    } else {
                        child.is_eol = false;
                        child_pointer = child.next;
                        line_preserve_width += node.row_gap;
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
                    preserve_width = preserve_width.max(child_resolved.rect.width);
                    preserve_height += child_resolved.rect.height;

                    if child.next.is_null() {
                        child.is_eol = true;
                        break;
                    } else {
                        child.is_eol = false;
                        child_pointer = child.next;
                        preserve_height += node.col_gap;
                    }
                }
            }
            WrapMode::Wrap | WrapMode::WrapReverse => {
                let mut line_preserve_width = 0f32;
                let mut line_preserve_height = 0f32;

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

                    if inner_height_constraint < line_preserve_height + child_resolved.rect.height {
                        let previous_child = unsafe { &mut *child.prev };
                        previous_child.is_eol = true;

                        preserve_width += line_preserve_width;
                        preserve_height = preserve_height.max(line_preserve_height);

                        if !child.next.is_null() {
                            preserve_width += node.row_gap;
                            child_pointer = child.next;

                            line_preserve_width = child_resolved.rect.width;
                            line_preserve_height = child_resolved.rect.height;
                            continue;
                        } else {
                            preserve_width += child_resolved.rect.width;
                            preserve_height = preserve_height.max(child_resolved.rect.height);

                            break;
                        }
                    }

                    line_preserve_width = line_preserve_width.max(child_resolved.rect.width);
                    line_preserve_height += child_resolved.rect.height;

                    if child.next.is_null() {
                        child.is_eol = true;

                        preserve_width += line_preserve_width;
                        preserve_height = preserve_height.max(line_preserve_height);

                        break;
                    } else {
                        child.is_eol = false;
                        child_pointer = child.next;
                        line_preserve_height += node.col_gap;
                    }
                }
            }
        },
    }

    let self_resolved = unsafe { &mut *node.resolved };

    match node.width {
        Some(_) => {
            if (self_resolved.rect.width - outer_width_constraint).abs() > 0.01 {
                self_resolved.is_width_changed = true;
                self_resolved.rect.width = outer_width_constraint;
            }
        }
        None => {
            let mut self_width = preserve_width + node.padding.left + node.padding.right;

            if let Some(max_width) = node.max_width {
                self_width = max_width.resolve(constraint.max_width).min(self_width);
            }
            if let Some(min_width) = node.min_width {
                self_width = min_width.resolve(constraint.max_width).min(self_width);
            }
            self_width = constraint.max_width.min(self_width);

            if (self_resolved.rect.width - self_width).abs() > 0.01 {
                self_resolved.is_width_changed = true;
                self_resolved.rect.width = self_width;
            }
        }
    }
    match node.height {
        Some(_) => {
            if (self_resolved.rect.height - outer_height_constraint).abs() > 0.01 {
                self_resolved.is_height_changed = true;
                self_resolved.rect.height = outer_height_constraint;
            }
        }
        None => {
            let mut self_height = preserve_height + node.padding.top + node.padding.bottom;

            if let Some(max_height) = node.max_height {
                self_height = max_height.resolve(constraint.max_height).min(self_height);
            }
            if let Some(min_height) = node.min_height {
                self_height = min_height.resolve(constraint.max_height).min(self_height);
            }
            self_height = constraint.max_height.min(self_height);

            if (self_resolved.rect.height - self_height).abs() > 0.01 {
                self_resolved.is_height_changed = true;
                self_resolved.rect.height = self_height;
            }
        }
    }
}

fn compute_flex_offset(node: &mut FlexLayoutNode, offset: Size) {}
