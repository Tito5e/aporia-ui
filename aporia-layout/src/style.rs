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
    // 左端から揃えていく
    Start,
    // 右端から揃えていく
    End,
    // unsafe-centerに該当する動作を行う、safe-centerは将来的に実装する
    Center,
    // 仮に範囲からはみ出す場合、Startにfallbackする
    SafeCenter,
    // 要素の中間部分のみに余白を配分する
    SpaceBetween,
    // 要素の左右に余白を配分する
    SpaceAround,
    // 要素の左右に余白を配分するが、要素の左端と右端、そして中間の余白が同一になるようにする
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
}
