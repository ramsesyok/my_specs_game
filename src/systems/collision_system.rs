// src/systems/collision.rs
//
// このファイルでは、テーブル境界との衝突処理と、
// ボール同士の衝突判定および反発処理を３つのフェーズに分割して実装します。

use crate::components::{Ball, Position, Table, Velocity};
use specs::prelude::*;
use specs::Entity;

/// CollisionSystem は、各シミュレーションステップにおいて、
/// 1. テーブル境界との衝突処理、
/// 2. ボール同士の衝突判定および反発処理（ペアごと、i < j）
/// を順次実施します。
pub struct CollisionSystem;

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Ball>,
        ReadStorage<'a, Table>,
    );

    fn run(&mut self, (entities, mut pos, mut vel, ball, table_storage): Self::SystemData) {
        // フェーズ1: テーブル（ビリヤード台）の境界との衝突判定と反射処理
        if let Some(table) = (&table_storage).join().next() {
            Self::process_table_collisions(&mut pos, &mut vel, &ball, table);
        }
        // フェーズ2および3: ボール同士の衝突判定および反発処理をペアごとに実施
        Self::process_ball_collisions(&entities, &mut pos, &mut vel, &ball);
    }
}

impl CollisionSystem {
    /// 【フェーズ1】
    /// 各ボールについて、テーブル境界との衝突判定と反射処理を行います。
    /// この関数は、各ボールの状態を引数として受け取り、handle_table_collision() という純粋関数を呼び出して結果を反映します。
    fn process_table_collisions(
        pos: &mut WriteStorage<Position>,
        vel: &mut WriteStorage<Velocity>,
        ball: &ReadStorage<Ball>,
        table: &Table,
    ) {
        for (p, v, b) in (pos, vel, ball).join() {
            // 純粋関数 handle_table_collision() で新しい位置と速度を計算
            let (new_pos, new_vel) = Self::handle_table_collision(*p, *v, b, table);
            *p = new_pos;
            *v = new_vel;
        }
    }

    /// テーブルとの衝突処理を行う純粋関数
    /// 入力値（位置、速度、ボールの諸元、テーブル情報）から、衝突判定を行い、
    /// 必要に応じて反射処理後の新しい状態を返します。
    fn handle_table_collision(
        pos: Position,
        vel: Velocity,
        ball: &Ball,
        table: &Table,
    ) -> (Position, Velocity) {
        let mut new_pos = pos;
        let mut new_vel = vel;

        // 左側の壁との衝突
        if new_pos.x - ball.radius < 0.0 {
            new_pos.x = ball.radius;
            new_vel.x = -new_vel.x * ball.restitution;
        }
        // 右側の壁との衝突
        if new_pos.x + ball.radius > table.width {
            new_pos.x = table.width - ball.radius;
            new_vel.x = -new_vel.x * ball.restitution;
        }
        // 下側の壁との衝突
        if new_pos.y - ball.radius < 0.0 {
            new_pos.y = ball.radius;
            new_vel.y = -new_vel.y * ball.restitution;
        }
        // 上側の壁との衝突
        if new_pos.y + ball.radius > table.height {
            new_pos.y = table.height - ball.radius;
            new_vel.y = -new_vel.y * ball.restitution;
        }

        (new_pos, new_vel)
    }

    /// 【フェーズ2 & 3】
    /// ボール同士の衝突判定および反発処理を、すべてのボールについてペアごと（i < j）に実施します。
    /// 各ボールの情報を収集し、compute_ball_collision_impulse() という純粋関数で各ペアの衝突判定とインパルス計算を行い、
    /// 結果として得られた衝突インパルスを各ボールの速度に反映します。
    fn process_ball_collisions(
        entities: &Entities,
        pos: &mut WriteStorage<Position>,
        vel: &mut WriteStorage<Velocity>,
        ball: &ReadStorage<Ball>,
    ) {
        // 以下のブロック内で、pos と vel の不変借用を行い、ball_info を収集する
        let ball_info: Vec<_> = {
            let pos_ref = &*pos;
            let vel_ref = &*vel;
            (&*entities, pos_ref, vel_ref, ball)
                .join()
                .map(|(ent, p, v, b)| (ent, p.x, p.y, v.x, v.y, b.mass, b.restitution, b.radius))
                .collect()
        };

        // i < j となるように、全ペアについて衝突判定を実施
        for i in 0..ball_info.len() {
            for j in (i + 1)..ball_info.len() {
                if let Some((impulse_x, impulse_y)) =
                    Self::compute_ball_collision_impulse(&ball_info[i], &ball_info[j])
                {
                    let (entity_a, _, _, _, _, mass_a, _, _) = ball_info[i];
                    let (entity_b, _, _, _, _, mass_b, _, _) = ball_info[j];

                    // 衝突インパルスを各ボールの速度に反映
                    if let Some(va) = vel.get_mut(entity_a) {
                        va.x += impulse_x / mass_a;
                        va.y += impulse_y / mass_a;
                    }
                    if let Some(vb) = vel.get_mut(entity_b) {
                        vb.x -= impulse_x / mass_b;
                        vb.y -= impulse_y / mass_b;
                    }
                }
            }
        }
    }

    /// 【フェーズ2：個々のペアごとの衝突判定】
    /// ボール A とボール B の情報から、衝突が発生している場合のインパルス（反発）を計算する純粋関数です。
    ///
    /// 入力タプルの内容は次のとおりです:
    /// - (Entity, pos_x, pos_y, vel_x, vel_y, mass, restitution, radius)
    ///
    /// 衝突している場合、(impulse_x, impulse_y) を返します。
    /// 衝突していない場合は None を返します。
    fn compute_ball_collision_impulse(
        a: &(Entity, f32, f32, f32, f32, f32, f32, f32),
        b: &(Entity, f32, f32, f32, f32, f32, f32, f32),
    ) -> Option<(f32, f32)> {
        // a: (entity, pos_x, pos_y, vel_x, vel_y, mass, restitution, radius)
        // b: (entity, pos_x, pos_y, vel_x, vel_y, mass, restitution, radius)
        let dx = b.1 - a.1;
        let dy = b.2 - a.2;
        let dist_sq = dx * dx + dy * dy;
        let radius_sum = a.7 + b.7; // 各ボールの半径の和

        // 衝突していなければ、または完全に重なっている場合は何も返さない
        if dist_sq >= radius_sum * radius_sum || dist_sq == 0.0 {
            return None;
        }

        let distance = dist_sq.sqrt();
        let nx = dx / distance;
        let ny = dy / distance;

        // 相対速度（a の速度 - b の速度）
        let rvx = a.3 - b.3;
        let rvy = a.4 - b.4;
        let vel_along_normal = rvx * nx + rvy * ny;

        // すでに分離している場合は何もしない
        if vel_along_normal > 0.0 {
            return None;
        }

        // 反発係数は両者のうち小さい方を採用
        let e = a.6.min(b.6);

        // インパルスの大きさを計算
        let impulse_mag = -(1.0 + e) * vel_along_normal / (1.0 / a.5 + 1.0 / b.5);

        Some((impulse_mag * nx, impulse_mag * ny))
    }
}
