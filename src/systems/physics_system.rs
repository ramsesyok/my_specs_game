// src/systems/physics.rs
//
// このファイルでは、各エンティティの速度情報をもとに位置を更新する物理シミュレーション（PhysicsSystem）を実装します。

use crate::components::{Position, Velocity};
use crate::TimeDelta;
use specs::prelude::*;

/// PhysicsSystem は、各エンティティの位置を速度に基づいて更新します。
pub struct PhysicsSystem;

impl<'a> System<'a> for PhysicsSystem {
    type SystemData = (
        WriteStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
        Read<'a, TimeDelta>,
    );

    fn run(&mut self, (mut pos, vel, time): Self::SystemData) {
        let dt = time.dt.as_secs_f32();
        // オイラー法により、すべての対象エンティティの位置を更新します。
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.x += vel.x * dt;
            pos.y += vel.y * dt;
        }
    }
}
