use crate::geometry::{Dimension, Padding};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    Start,
    End,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatioMode {
    Width,
    Height,
    Fit,
    Fill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexPlacement {
    Start,
    End,
    Center, // unsafe-centerに該当する動作を行う、safe-centerは将来的に実装する
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapMode {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sizing {
    Fit {
        min_width: Option<Dimension>,
        min_height: Option<Dimension>,
        max_width: Option<Dimension>,
        max_height: Option<Dimension>,
    },
    Own {
        width: Option<Dimension>,
        height: Option<Dimension>,
        min_width: Option<Dimension>,
        min_height: Option<Dimension>,
        max_width: Option<Dimension>,
        max_height: Option<Dimension>,
    },
    Ratio {
        mode: RatioMode,
        ratio: f32,
        max_width: Option<Dimension>,
        max_height: Option<Dimension>,
    },
}

impl Sizing {
    pub fn fit() -> Self {
        Self::Fit { min_width: None, min_height: None, max_width: None, max_height: None }
    }

    pub fn px(width: f32, height: f32) -> Self {
        Self::Own {
            width: Some(Dimension::Px(width)),
            height: Some(Dimension::Px(height)),
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxStyle {
    pub sizing: Sizing,
    pub padding: Padding,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlexStyle {
    pub sizing: Sizing,
    pub padding: Padding,
    pub direction: Direction,
    pub wrap_mode: WrapMode,
    pub row_placement: FlexPlacement,
    pub col_placement: FlexPlacement,
    pub row_gap: f32,
    pub col_gap: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridStyle {
    pub sizing: Sizing,
    pub padding: Padding,
    pub direction: Direction,
    pub row_gap: f32,
    pub col_gap: f32,
    pub default_row_track: Dimension,
    pub default_col_track: Dimension,
}
