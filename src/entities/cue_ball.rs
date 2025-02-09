// src/entities/cue_ball.rs
//
// このファイルでは、手球（cue ball）のエンティティを生成する関数を定義します。

use crate::components::{Ball, Position, Velocity};
use crate::config::Config;
use specs::prelude::*;

/// 手球エンティティを生成する関数です。
///
/// # 引数
/// - `world`: ECS の World への可変参照
/// - `config`: 設定情報
///
/// # 戻り値
/// 生成されたエンティティを返します。
pub fn create_cue_ball(world: &mut World, config: &Config) -> Entity {
    world
        .create_entity()
        .with(Position {
            x: config.cue_ball.x,
            y: config.cue_ball.y,
        })
        .with(Velocity {
            // m/s から cm/s への変換（×100）
            x: config.cue_ball.vx * 100.0,
            y: config.cue_ball.vy * 100.0,
        })
        .with(Ball {
            radius: config.ball.radius,
            mass: config.ball.mass,
            restitution: config.ball.restitution,
        })
        .build()
}
