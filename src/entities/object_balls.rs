// src/entities/object_balls.rs
//
// このファイルでは、的球（object balls）のエンティティを生成する関数を定義します。

use crate::components::{Ball, Position, Velocity};
use crate::config::Config;
use specs::prelude::*;

/// 的球エンティティを生成する関数です。
///
/// # 引数
/// - `world`: ECS の World への可変参照
/// - `config`: 設定情報
///
/// # 戻り値
/// 生成されたエンティティの Vec を返します。
pub fn create_object_balls(world: &mut World, config: &Config) -> Vec<Entity> {
    let mut entities = Vec::new();
    // config.object_balls.positions に記載された各座標で的球を生成します。
    for pos_config in &config.object_balls.positions {
        let entity = world
            .create_entity()
            .with(Position {
                x: pos_config.x,
                y: pos_config.y,
            })
            .with(Velocity { x: 0.0, y: 0.0 })
            .with(Ball {
                radius: config.ball.radius,
                mass: config.ball.mass,
                restitution: config.ball.restitution,
            })
            .build();
        entities.push(entity);
    }
    entities
}
