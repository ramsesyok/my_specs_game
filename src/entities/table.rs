// entities.rs
use crate::components::table::Table;
use crate::config::Config;
use specs::prelude::*;

/// テーブル（ビリヤード台）のエンティティを生成する関数です。
///
/// # 引数
/// - `world`: ECS の World への可変参照
/// - `config`: 設定情報（テーブルサイズ・ヘッドスポットの座標など）
pub fn create_table(world: &mut World, config: &Config) -> Entity {
    // テーブルは移動しないため、位置情報はヘッドスポットの値のみ保持します。
    world
        .create_entity()
        .with(Table {
            width: config.table.width,
            height: config.table.height,
        })
        .build()
}
