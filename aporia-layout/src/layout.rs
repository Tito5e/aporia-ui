use crate::geometry::Constraint;
use crate::node::{BoxLayoutNode, FlexLayoutNode, LayoutNode, LayoutNodeExt};
use crate::style::{Direction, FlexPlacement, Placement, RatioMode, Sizing, WrapMode};
use aporia_core::geometry::Size;

pub fn compute_layout(node: &mut LayoutNode, constraint: Constraint) {
    // PASS-1
}

fn compute_size(node: &mut LayoutNode, constraint: Constraint) {
    // RESET DIRTY FLAGS
    node.clear_dirty();

    match node {
        LayoutNode::Box(node) => {
            compute_box_layout(node, constraint);
        }
    }
}

fn compute_offset(node: &mut LayoutNode, offset: Size) {}

fn compute_box_layout(node: &mut BoxLayoutNode, constraint: Constraint) {
    let (w, h): (f32, f32) = match node.sizing {
        Sizing::Own { width, height, min_width, max_width, min_height, max_height } => {
            let mut w = width.map(|d| d.resolve(constraint.max_width)).unwrap_or(0.0);
            let mut h = height.map(|d| d.resolve(constraint.max_height)).unwrap_or(0.0);

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
                compute_layout(
                    unsafe { &mut *node.child },
                    Constraint::new(inner_w, inner_h),
                    Size::new(offset.width + node.padding.left, offset.height + node.padding.top),
                );
            }

            (w, h)
        }
        Sizing::Fit { min_width, min_height, max_width, max_height } => {
            let inner_w = constraint.max_width - node.padding.left - node.padding.right;
            let inner_h = constraint.max_height - node.padding.top - node.padding.bottom;

            let mut self_w;
            let mut self_h;

            if !node.child.is_null() {
                let child = unsafe { &mut *node.child };
                compute_layout(child, Constraint::new(inner_w, inner_h));

                // オフセットメモ Size::new(offset.width + node.padding.left, offset.height + node.padding.top),

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

            /*if (self_resolved.rect.x - offset.width).abs() > 0.01 {
                self_resolved.is_x_changed = true;
                self_resolved.rect.x = offset.width;
            }
            if (self_resolved.rect.y - offset.height).abs() > 0.01 {
                self_resolved.is_y_changed = true;
                self_resolved.rect.y = offset.height;
            }*/

            (self_w, self_h)
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
                    compute_layout(unsafe { &mut *node.child }, Constraint::new(inner_w, inner_h));
                }

                (w, h)
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
                    compute_layout(unsafe { &mut *node.child }, Constraint::new(inner_w, inner_h));
                }

                (w, h)
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
                    compute_layout(unsafe { &mut *node.child }, Constraint::new(inner_w, inner_h));
                }

                (w, h)
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
                    compute_layout(unsafe { &mut *node.child }, Constraint::new(inner_w, inner_h));
                }

                (w, h)
            }
        },
    };

    // PASS-2: Offset (for Box) 別パスに移動する
    /*if !node.child.is_null() {
        match node.row_placement {
            Placement::Start => {
                let inner_resolved = unsafe { &mut *node.child }.resolved_mut();

                if (inner_resolved.rect.x - (offset.width + node.padding.left)).abs() > 0.01 {
                    inner_resolved.is_x_changed = true;
                    inner_resolved.rect.x = offset.width + node.padding.left;
                }
            }
            Placement::End => {
                let inner_resolved = unsafe { &mut *node.child }.resolved_mut();

                if (inner_resolved.rect.x
                    - (offset.width + w - node.padding.right - inner_resolved.rect.width))
                    .abs()
                    > 0.01
                {
                    inner_resolved.is_x_changed = true;
                    inner_resolved.rect.x =
                        offset.width + w - node.padding.right - inner_resolved.rect.width;
                }
            }
            Placement::Center => {
                let inner_resolved = unsafe { &mut *node.child }.resolved_mut();

                if (inner_resolved.rect.x
                    - (offset.width + node.padding.left + offset.width + w - node.padding.right)
                        / 2f32)
                    .abs()
                    > 0.01
                {
                    inner_resolved.is_x_changed = true;
                    inner_resolved.rect.x = (offset.width + node.padding.left + offset.width + w
                        - node.padding.right)
                        / 2f32;
                }
            }
        }
        match node.col_placement {
            Placement::Start => {
                let inner_resolved = unsafe { &mut *node.child }.resolved_mut();

                if (inner_resolved.rect.y - (offset.height + node.padding.top)).abs() > 0.01 {
                    inner_resolved.is_y_changed = true;
                    inner_resolved.rect.y = offset.height + node.padding.top;
                }
            }
            Placement::End => {
                let inner_resolved = unsafe { &mut *node.child }.resolved_mut();

                if (inner_resolved.rect.y
                    - (offset.height + h - node.padding.bottom - inner_resolved.rect.height))
                    .abs()
                    > 0.01
                {
                    inner_resolved.is_y_changed = true;
                    inner_resolved.rect.y =
                        offset.height + h - node.padding.bottom - inner_resolved.rect.height;
                }
            }
            Placement::Center => {
                let inner_resolved = unsafe { &mut *node.child }.resolved_mut();

                if (inner_resolved.rect.y
                    - (offset.height + node.padding.top + offset.height + h - node.padding.bottom)
                        / 2f32)
                    .abs()
                    > 0.01
                {
                    inner_resolved.is_y_changed = true;
                    inner_resolved.rect.y = (offset.height + node.padding.top + offset.height + h
                        - node.padding.bottom)
                        / 2f32;
                }
            }
        }
    }*/
}

fn compute_flex_layout(node: &mut FlexLayoutNode, constraint: Constraint) {
    match node.sizing {
        Sizing::Fit { min_width, min_height, max_width, max_height } => match node.direction {
            Direction::Row => {
                let inner_w = constraint.max_width - node.padding.left - node.padding.right;
                let inner_h = constraint.max_height - node.padding.top - node.padding.bottom;

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

                            current_child_pointer = current_child.next;
                            if current_child_pointer.is_null() {
                                break;
                            } else {
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

                        self_w = constraint.max_width.min(self_w);
                        self_h = constraint.max_height.min(self_h);

                        // OFFSET PASS
                        match node.row_placement {
                            FlexPlacement::Start => {}
                            FlexPlacement::End => {}
                            FlexPlacement::Center => {}
                            FlexPlacement::SpaceBetween => {}
                            FlexPlacement::SpaceAround => {}
                            FlexPlacement::SpaceEvenly => {}
                        }

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
                    WrapMode::Wrap => {}
                    WrapMode::WrapReverse => {}
                }
            }
            Direction::Column => {}
        },
        Sizing::Own { .. } => {}
        Sizing::Ratio { .. } => {}
    }
}
