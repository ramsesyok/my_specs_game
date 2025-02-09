// components/ball.rs
use specs::prelude::*;

/// ボールの物理特性を保持するコンポーネントです。
#[derive(Debug, Copy, Clone)]
pub struct Ball {
    /// ボールの半径（cm）
    pub radius: f32,
    /// ボールの質量（g）
    pub mass: f32,
    /// 反発係数（衝突後の反発の大きさ）
    pub restitution: f32,
}

// Component トレイトの実装。VecStorage を用います。
impl Component for Ball {
    type Storage = VecStorage<Self>;
}
