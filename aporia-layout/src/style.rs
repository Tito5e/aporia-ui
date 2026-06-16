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

#[derive(Debug, Clone, PartialEq)]
pub struct BoxStyle {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub min_width: Option<Dimension>,
    pub min_height: Option<Dimension>,
    pub max_width: Option<Dimension>,
    pub max_height: Option<Dimension>,

    pub padding: Padding,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RatioStyle {
    pub mode: RatioMode,
    pub ratio: f32,
    pub max_width: Option<Dimension>,
    pub max_height: Option<Dimension>,

    pub padding: Padding,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlexStyle {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub min_width: Option<Dimension>,
    pub min_height: Option<Dimension>,
    pub max_width: Option<Dimension>,
    pub max_height: Option<Dimension>,

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
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub min_width: Option<Dimension>,
    pub min_height: Option<Dimension>,
    pub max_width: Option<Dimension>,
    pub max_height: Option<Dimension>,

    pub padding: Padding,
    pub direction: Direction,
    pub row_gap: f32,
    pub col_gap: f32,
    pub default_row_track: Dimension,
    pub default_col_track: Dimension,
}
