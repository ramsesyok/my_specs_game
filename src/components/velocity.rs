// components/velocity.rs
use specs::prelude::*;

/// 2D 空間上の速度（移動量）を表すコンポーネントです。
#[derive(Debug, Copy, Clone)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

// Component トレイトを実装し、VecStorage を用います。
impl Component for Velocity {
    type Storage = VecStorage<Self>;
}
