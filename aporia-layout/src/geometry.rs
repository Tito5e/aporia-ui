#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Padding {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
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
}

impl Constraint {
    #[inline(always)]
    pub fn new(max_width: f32, max_height: f32) -> Self {
        Self { max_width, max_height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Percent(f32),
    Px(f32),
    Fr(f32),
}

impl Dimension {
    pub fn resolve(&self, constraint: f32) -> f32 {
        match self {
            Dimension::Percent(percent) => constraint * percent,
            Dimension::Px(pixel) => *pixel,
            // Dimension::Frは相対単位なので制約ベースでは解決できない
            // 各種レイアウトロジックにおいて特殊実装を持つ
            // また、width, heightなどではそもそも指定の意味を持たない
            // あくまでFlexやGridのTrack指定でのみ意味を持つため、resolveにおいては制約をそのまま返す
            Dimension::Fr(_) => constraint,
        }
    }
}
