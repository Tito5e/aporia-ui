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
            // TODO: Frに対して渡すConstraintのみFr(1)の単位長さにすることで統一的に扱える可能性がある
            Dimension::Fr(fr) => constraint * fr,
        }
    }
}
