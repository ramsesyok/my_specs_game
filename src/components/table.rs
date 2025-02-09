// components/table.rs
use specs::prelude::*;

/// ビリヤード台を表すコンポーネントです。
#[derive(Debug, Copy, Clone)]
pub struct Table {
    /// テーブルの横幅（cm）
    pub width: f32,
    /// テーブルの高さ（cm）
    pub height: f32,
}

// Component トレイトの実装。VecStorage を用います。
impl Component for Table {
    type Storage = VecStorage<Self>;
}
