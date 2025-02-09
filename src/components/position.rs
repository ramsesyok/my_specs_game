// components/position.rs
use specs::prelude::*;

/// 2D 空間上の位置を表すコンポーネントです。
#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

// ECS の Component トレイトを実装し、VecStorage を用います。
impl Component for Position {
    type Storage = VecStorage<Self>;
}
