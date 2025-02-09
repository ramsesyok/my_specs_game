// src/systems/print.rs
//
// このファイルでは、各エンティティ（ボール）の現在の位置を
// tracing クレートを用いたログ出力により表示する PrintSystem を実装します。

use crate::components::{Ball, Position};
use specs::prelude::*;
use tracing::info;

/// PrintSystem は、各ボールの位置情報をログ出力します。
pub struct LoggingSystem;

impl<'a> System<'a> for LoggingSystem {
    type SystemData = (ReadStorage<'a, Position>, ReadStorage<'a, Ball>);

    fn run(&mut self, (pos, ball): Self::SystemData) {
        // Position と Ball コンポーネントを持つすべてのエンティティについて位置をログ出力します。
        for (pos, _ball) in (&pos, &ball).join() {
            info!("Ball position: ({:.2}, {:.2})", pos.x, pos.y);
        }
    }
}
