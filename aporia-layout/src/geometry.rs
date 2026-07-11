/// 要素のpaddingを表現します
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Padding {
    /// padding-topに該当します
    pub top: f32,
    /// padding-bottomに該当します
    pub bottom: f32,
    /// padding-leftに該当します
    pub left: f32,
    /// padding-rightに該当します
    pub right: f32,
}

impl Padding {
    #[inline(always)]
    pub fn tblr(top: f32, bottom: f32, left: f32, right: f32) -> Self {
        Self { top, bottom, left, right }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraint {
    pub max_width: f32,
    pub max_height: f32,
    pub fr_width: Option<f32>,
    pub fr_height: Option<f32>,
}

impl Constraint {
    #[inline(always)]
    pub fn new(max_width: f32, max_height: f32) -> Self {
        Self { max_width, max_height, fr_width: None, fr_height: None }
    }

    #[inline(always)]
    pub fn new_fr(max_width: f32, max_height: f32, fr_width: f32, fr_height: f32) -> Self {
        Self { max_width, max_height, fr_width: Some(fr_width), fr_height: Some(fr_height) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Percent(f32),
    Px(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridDimension {
    Percent(f32),
    Px(f32),
    Fr(f32),
}

impl Dimension {
    pub fn resolve(&self, constraint: f32) -> f32 {
        match self {
            Dimension::Percent(percent) => constraint * percent,
            Dimension::Px(pixel) => *pixel,
        }
    }
}

impl GridDimension {
    pub fn resolve(&self, constraint: f32, fr_unit: Option<f32>) -> f32 {
        match self {
            GridDimension::Percent(percent) => constraint * percent,
            GridDimension::Px(pixel) => *pixel,

            GridDimension::Fr(fr) => {
                if let Some(fr_unit) = fr_unit {
                    fr_unit * fr
                } else {
                    constraint * fr
                }
            }
        }
    }
}
