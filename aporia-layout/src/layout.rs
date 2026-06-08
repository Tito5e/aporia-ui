use crate::geometry::Constraint;
use crate::node::{BoxLayoutNode, FlexLayoutNode, LayoutNode, LayoutNodeExt};
use crate::style::{Direction, FlexPlacement, Placement, RatioMode, Sizing, WrapMode};
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
        LayoutNode::Flex(node) => {
            compute_flex_offset(node, offset);
        }
    }
}

fn compute_box_size(node: &mut BoxLayoutNode, constraint: Constraint) {
    match node.sizing {
        Sizing::Own { width, height, min_width, max_width, min_height, max_height } => {
            let mut w = width.resolve(constraint.max_width);
            let mut h = height.resolve(constraint.max_height);

            if let Some(max_w) = max_width {
                w = max_w.resolve(constraint.max_width).min(w);
            }
            if let Some(max_h) = max_height {
                h = max_h.resolve(constraint.max_height).min(h);
            }

            if let Some(min_w) = min_width {
                w = min_w.resolve(constraint.max_width).max(w);
            }
            if let Some(min_h) = min_height {
                h = min_h.resolve(constraint.max_height).max(h);
            }

            {
                let self_resolved = unsafe { &mut *node.resolved };
                if (self_resolved.rect.width - w).abs() > 0.01 {
                    self_resolved.is_width_changed = true;
                    self_resolved.rect.width = w;
                }
                if (self_resolved.rect.height - h).abs() > 0.01 {
                    self_resolved.is_height_changed = true;
                    self_resolved.rect.height = h;
                }
            }

            let inner_w = w - node.padding.left - node.padding.right;
            let inner_h = h - node.padding.top - node.padding.bottom;

            if !node.child.is_null() {
                compute_size(unsafe { &mut *node.child }, Constraint::new(inner_w, inner_h));
            }
        }
        Sizing::Fit { min_width, min_height, max_width, max_height } => {
            let inner_w = constraint.max_width - node.padding.left - node.padding.right;
            let inner_h = constraint.max_height - node.padding.top - node.padding.bottom;

            let mut self_w;
            let mut self_h;

            if !node.child.is_null() {
                let child = unsafe { &mut *node.child };
                compute_size(child, Constraint::new(inner_w, inner_h));

                let inner_resolved = child.resolved_mut();

                self_w = inner_resolved.rect.width + node.padding.left + node.padding.right;
                self_h = inner_resolved.rect.height + node.padding.top + node.padding.bottom;
            } else {
                self_w = node.padding.left + node.padding.right;
                self_h = node.padding.top + node.padding.bottom;
            }

            if let Some(max_w) = max_width {
                self_w = max_w.resolve(constraint.max_width).min(self_w);
            }
            if let Some(max_h) = max_height {
                self_h = max_h.resolve(constraint.max_height).min(self_h);
            }

            if let Some(min_w) = min_width {
                self_w = min_w.resolve(constraint.max_width).max(self_w);
            }
            if let Some(min_h) = min_height {
                self_h = min_h.resolve(constraint.max_height).max(self_h);
            }

            self_w = constraint.max_width.min(self_w);
            self_h = constraint.max_height.min(self_h);

            let self_resolved = unsafe { &mut *node.resolved };
            if (self_resolved.rect.width - self_w).abs() > 0.01 {
                self_resolved.is_width_changed = true;
                self_resolved.rect.width = self_w;
            }
            if (self_resolved.rect.height - self_h).abs() > 0.01 {
                self_resolved.is_height_changed = true;
                self_resolved.rect.height = self_h;
            }
        }
        Sizing::Ratio { mode, ratio, max_width, max_height } => match mode {
            RatioMode::Width => {
                let max_w = max_width
                    .map(|w| w.resolve(constraint.max_width))
                    .unwrap_or(0.0)
                    .min(constraint.max_width);

                let w = max_w;
                let h = max_w / ratio;

                let inner_w = w - node.padding.left - node.padding.right;
                let inner_h = h - node.padding.top - node.padding.bottom;

                if !node.child.is_null() {
                    compute_size(unsafe { &mut *node.child }, Constraint::new(inner_w, inner_h));
                }
            }
            RatioMode::Height => {
                let max_h = max_height
                    .map(|h| h.resolve(constraint.max_height))
                    .unwrap_or(0.0)
                    .min(constraint.max_height);

                let w = max_h;
                let h = max_h * ratio;

                let inner_w = w - node.padding.left - node.padding.right;
                let inner_h = h - node.padding.top - node.padding.bottom;

                if !node.child.is_null() {
                    compute_size(unsafe { &mut *node.child }, Constraint::new(inner_w, inner_h));
                }
            }
            RatioMode::Fit => {
                let max_w = max_width
                    .map(|w| w.resolve(constraint.max_width))
                    .unwrap_or(0.0)
                    .min(constraint.max_width);
                let max_h = max_height
                    .map(|h| h.resolve(constraint.max_height))
                    .unwrap_or(0.0)
                    .min(constraint.max_height);

                let constraint_ratio = max_w / max_h;

                let (w, h) = if constraint_ratio > ratio {
                    let w = max_h;
                    let h = max_h * ratio;

                    (w, h)
                } else {
                    let w = max_w;
                    let h = max_w / ratio;

                    (w, h)
                };

                let inner_w = w - node.padding.left - node.padding.right;
                let inner_h = h - node.padding.top - node.padding.bottom;

                if !node.child.is_null() {
                    compute_size(unsafe { &mut *node.child }, Constraint::new(inner_w, inner_h));
                }
            }
            RatioMode::Fill => {
                let max_w = max_width
                    .map(|w| w.resolve(constraint.max_width))
                    .unwrap_or(0.0)
                    .min(constraint.max_width);
                let max_h = max_height
                    .map(|h| h.resolve(constraint.max_height))
                    .unwrap_or(0.0)
                    .min(constraint.max_height);

                let constraint_ratio = max_w / max_h;

                let (w, h) = if constraint_ratio < ratio {
                    let w = max_h;
                    let h = max_h * ratio;

                    (w, h)
                } else {
                    let w = max_w;
                    let h = max_w / ratio;

                    (w, h)
                };

                let inner_w = w - node.padding.left - node.padding.right;
                let inner_h = h - node.padding.top - node.padding.bottom;

                if !node.child.is_null() {
                    compute_size(unsafe { &mut *node.child }, Constraint::new(inner_w, inner_h));
                }
            }
        },
    };
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
    match node.sizing {
        Sizing::Fit { min_width, min_height, max_width, max_height } => match node.direction {
            Direction::Row => {
                let inner_w = if let Some(max_width) = max_width {
                    max_width.resolve(constraint.max_width) - node.padding.left - node.padding.right
                } else {
                    constraint.max_width - node.padding.left - node.padding.right
                };
                let inner_h = if let Some(max_height) = max_height {
                    max_height.resolve(constraint.max_height)
                        - node.padding.top
                        - node.padding.bottom
                } else {
                    constraint.max_height - node.padding.top - node.padding.bottom
                };

                match node.wrap_mode {
                    WrapMode::NoWrap => {
                        let mut current_preserve_w = 0f32;
                        let mut current_preserve_h = 0f32;

                        let mut current_child_pointer = node.child_head;

                        // サイズ確定+集計パス
                        loop {
                            let current_child = unsafe { &mut *current_child_pointer };
                            {
                                let current_child_node = unsafe { &mut *current_child.child };
                                compute_layout(
                                    current_child_node,
                                    Constraint::new(inner_w, inner_h),
                                );
                            }
                            let current_child_node = unsafe { &mut *current_child.child };
                            let inner_resolved = current_child_node.resolved_mut();
                            current_preserve_w += inner_resolved.rect.width;
                            current_preserve_h = current_preserve_h.max(inner_resolved.rect.height);

                            if current_child.next.is_null() {
                                current_child.is_eol = true;
                                break;
                            } else {
                                current_child.is_eol = false;
                                current_child_pointer = current_child.next;
                                current_preserve_w += node.row_gap;
                            }
                        }

                        let mut self_w =
                            current_preserve_w + node.padding.left + node.padding.right;
                        let mut self_h =
                            current_preserve_h + node.padding.top + node.padding.bottom;

                        if let Some(max_w) = max_width {
                            self_w = max_w.resolve(constraint.max_width).min(self_w);
                        }
                        if let Some(max_h) = max_height {
                            self_h = max_h.resolve(constraint.max_height).min(self_h);
                        }

                        if let Some(min_w) = min_width {
                            self_w = min_w.resolve(constraint.max_width).max(self_w);
                        }
                        if let Some(min_h) = min_height {
                            self_h = min_h.resolve(constraint.max_height).max(self_h);
                        }

                        // TODO: Fitを親要素に合わせるべきか、子要素に合わせるべきか
                        // self_w = constraint.max_width.min(self_w);
                        // self_h = constraint.max_height.min(self_h);

                        let self_resolved = unsafe { &mut *node.resolved };
                        if (self_resolved.rect.width - self_w).abs() > 0.01 {
                            self_resolved.is_width_changed = true;
                            self_resolved.rect.width = self_w;
                        }
                        if (self_resolved.rect.height - self_h).abs() > 0.01 {
                            self_resolved.is_height_changed = true;
                            self_resolved.rect.height = self_h;
                        }
                    }
                    WrapMode::Wrap | WrapMode::WrapReverse => {
                        let mut accum_preserve_w = 0f32;
                        let mut accum_preserve_h = 0f32;

                        let mut current_preserve_w = 0f32;
                        let mut current_preserve_h = 0f32;

                        let mut current_child_pointer = node.child_head;

                        // サイズ確定+集計パス
                        loop {
                            let current_child = unsafe { &mut *current_child_pointer };
                            {
                                let current_child_node = unsafe { &mut *current_child.child };
                                compute_layout(
                                    current_child_node,
                                    Constraint::new(inner_w, inner_h),
                                );
                            }

                            let current_child_node = unsafe { &mut *current_child.child };
                            let inner_resolved = current_child_node.resolved_mut();

                            if inner_w < current_preserve_w + inner_resolved.rect.width {
                                current_child.is_eol = true;
                                current_child_pointer = current_child.next;

                                accum_preserve_w = accum_preserve_w.max(current_preserve_w);
                                accum_preserve_h += current_preserve_h;

                                if !current_child.next.is_null() {
                                    accum_preserve_h += node.col_gap;
                                }

                                current_preserve_w = 0f32;
                                current_preserve_h = 0f32;
                                continue;
                            }

                            current_preserve_w += inner_resolved.rect.width;
                            current_preserve_h = current_preserve_h.max(inner_resolved.rect.height);

                            if current_child.next.is_null() {
                                current_child.is_eol = true;

                                accum_preserve_w = accum_preserve_w.max(current_preserve_w);
                                accum_preserve_h += current_preserve_h;

                                break;
                            } else {
                                current_child.is_eol = false;
                                current_child_pointer = current_child.next;
                                current_preserve_w += node.row_gap;
                            }
                        }

                        let mut self_w = accum_preserve_w + node.padding.left + node.padding.right;
                        let mut self_h = accum_preserve_h + node.padding.top + node.padding.bottom;

                        if let Some(max_w) = max_width {
                            self_w = max_w.resolve(constraint.max_width).min(self_w);
                        }
                        if let Some(max_h) = max_height {
                            self_h = max_h.resolve(constraint.max_height).min(self_h);
                        }

                        if let Some(min_w) = min_width {
                            self_w = min_w.resolve(constraint.max_width).max(self_w);
                        }
                        if let Some(min_h) = min_height {
                            self_h = min_h.resolve(constraint.max_height).max(self_h);
                        }

                        // TODO: Fitを親要素に合わせるべきか、子要素に合わせるべきか
                        // self_w = constraint.max_width.min(self_w);
                        // self_h = constraint.max_height.min(self_h);

                        let self_resolved = unsafe { &mut *node.resolved };
                        if (self_resolved.rect.width - self_w).abs() > 0.01 {
                            self_resolved.is_width_changed = true;
                            self_resolved.rect.width = self_w;
                        }
                        if (self_resolved.rect.height - self_h).abs() > 0.01 {
                            self_resolved.is_height_changed = true;
                            self_resolved.rect.height = self_h;
                        }
                    }
                }
            }
            Direction::Column => {
                let inner_w = if let Some(max_width) = max_width {
                    max_width.resolve(constraint.max_width) - node.padding.left - node.padding.right
                } else {
                    constraint.max_width - node.padding.left - node.padding.right
                };
                let inner_h = if let Some(max_height) = max_height {
                    max_height.resolve(constraint.max_height)
                        - node.padding.top
                        - node.padding.bottom
                } else {
                    constraint.max_height - node.padding.top - node.padding.bottom
                };

                match node.wrap_mode {
                    WrapMode::NoWrap => {
                        let mut current_preserve_w = 0f32;
                        let mut current_preserve_h = 0f32;

                        let mut current_child_pointer = node.child_head;

                        // サイズ確定+集計パス
                        loop {
                            let current_child = unsafe { &mut *current_child_pointer };
                            {
                                let current_child_node = unsafe { &mut *current_child.child };
                                compute_layout(
                                    current_child_node,
                                    Constraint::new(inner_w, inner_h),
                                );
                            }
                            let current_child_node = unsafe { &mut *current_child.child };
                            let inner_resolved = current_child_node.resolved_mut();
                            current_preserve_w += inner_resolved.rect.width;
                            current_preserve_h = current_preserve_h.max(inner_resolved.rect.height);

                            if current_child.next.is_null() {
                                current_child.is_eol = true;
                                break;
                            } else {
                                current_child.is_eol = false;
                                current_child_pointer = current_child.next;
                                current_preserve_w += node.row_gap;
                            }
                        }

                        let mut self_w =
                            current_preserve_w + node.padding.left + node.padding.right;
                        let mut self_h =
                            current_preserve_h + node.padding.top + node.padding.bottom;

                        if let Some(max_w) = max_width {
                            self_w = max_w.resolve(constraint.max_width).min(self_w);
                        }
                        if let Some(max_h) = max_height {
                            self_h = max_h.resolve(constraint.max_height).min(self_h);
                        }

                        if let Some(min_w) = min_width {
                            self_w = min_w.resolve(constraint.max_width).max(self_w);
                        }
                        if let Some(min_h) = min_height {
                            self_h = min_h.resolve(constraint.max_height).max(self_h);
                        }

                        // TODO: Fitを親要素に合わせるべきか、子要素に合わせるべきか
                        // self_w = constraint.max_width.min(self_w);
                        // self_h = constraint.max_height.min(self_h);

                        let self_resolved = unsafe { &mut *node.resolved };
                        if (self_resolved.rect.width - self_w).abs() > 0.01 {
                            self_resolved.is_width_changed = true;
                            self_resolved.rect.width = self_w;
                        }
                        if (self_resolved.rect.height - self_h).abs() > 0.01 {
                            self_resolved.is_height_changed = true;
                            self_resolved.rect.height = self_h;
                        }
                    }
                    WrapMode::Wrap | WrapMode::WrapReverse => {
                        let mut accum_preserve_w = 0f32;
                        let mut accum_preserve_h = 0f32;

                        let mut current_preserve_w = 0f32;
                        let mut current_preserve_h = 0f32;

                        let mut current_child_pointer = node.child_head;

                        // サイズ確定+集計パス
                        loop {
                            let current_child = unsafe { &mut *current_child_pointer };
                            {
                                let current_child_node = unsafe { &mut *current_child.child };
                                compute_layout(
                                    current_child_node,
                                    Constraint::new(inner_w, inner_h),
                                );
                            }

                            let current_child_node = unsafe { &mut *current_child.child };
                            let inner_resolved = current_child_node.resolved_mut();

                            if inner_h < current_preserve_h + inner_resolved.rect.height {
                                current_child.is_eol = true;
                                current_child_pointer = current_child.next;

                                accum_preserve_w += current_preserve_w;
                                accum_preserve_h = accum_preserve_h.max(current_preserve_h);

                                if !current_child.next.is_null() {
                                    accum_preserve_w += node.row_gap;
                                }

                                current_preserve_w = 0f32;
                                current_preserve_h = 0f32;
                                continue;
                            }

                            current_preserve_w = current_preserve_w.max(inner_resolved.rect.width);
                            current_preserve_h += inner_resolved.rect.height;

                            if current_child.next.is_null() {
                                current_child.is_eol = true;

                                accum_preserve_w += current_preserve_w;
                                accum_preserve_h = accum_preserve_h.max(current_preserve_h);

                                break;
                            } else {
                                current_child.is_eol = false;
                                current_child_pointer = current_child.next;
                                current_preserve_h += node.col_gap;
                            }
                        }

                        let mut self_w = accum_preserve_w + node.padding.left + node.padding.right;
                        let mut self_h = accum_preserve_h + node.padding.top + node.padding.bottom;

                        if let Some(max_w) = max_width {
                            self_w = max_w.resolve(constraint.max_width).min(self_w);
                        }
                        if let Some(max_h) = max_height {
                            self_h = max_h.resolve(constraint.max_height).min(self_h);
                        }

                        if let Some(min_w) = min_width {
                            self_w = min_w.resolve(constraint.max_width).max(self_w);
                        }
                        if let Some(min_h) = min_height {
                            self_h = min_h.resolve(constraint.max_height).max(self_h);
                        }

                        // TODO: Fitを親要素に合わせるべきか、子要素に合わせるべきか
                        // self_w = constraint.max_width.min(self_w);
                        // self_h = constraint.max_height.min(self_h);

                        let self_resolved = unsafe { &mut *node.resolved };
                        if (self_resolved.rect.width - self_w).abs() > 0.01 {
                            self_resolved.is_width_changed = true;
                            self_resolved.rect.width = self_w;
                        }
                        if (self_resolved.rect.height - self_h).abs() > 0.01 {
                            self_resolved.is_height_changed = true;
                            self_resolved.rect.height = self_h;
                        }
                    }
                }
            }
        },
        Sizing::Own { width, height, min_width, min_height, max_width, max_height } => {}
        Sizing::Ratio { .. } => {}
    }
}
