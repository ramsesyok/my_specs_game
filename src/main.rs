// main.rs
// ここでは、各モジュール（config, entities, components, systems）を読み込みます。
use specs::prelude::*;
use std::error::Error;
use std::time::Duration;
use tracing;
use tracing_subscriber;

mod config;
// エンティティ生成モジュール：entities 以下に各関数を個別ファイルに分割
mod entities;
// components 以下の各ファイルをモジュールとして読み込みます。
mod components;

mod systems;

/// シミュレーションの時間刻み（dt）を保持するリソースです。
#[derive(Default)]
pub struct TimeDelta {
    pub dt: Duration,
}

fn main() -> Result<(), Box<dyn Error>> {
    // --- tracing の初期化 ---
    // ログ出力のために tracing_subscriber を初期化します。
    tracing_subscriber::fmt::init();

    // --- 1. YAML ファイルから設定情報を読み込みます ---
    // config.yaml に定義されたシミュレーションパラメータを読み込みます。
    let config = config::load_config("config.yaml").expect("Failed to load config.yaml");
    tracing::info!("Loaded configuration: {:?}", config);

    // --- 2. ECS の World を生成します ---
    let mut world = World::new();

    // --- 3. 各コンポーネントを World に登録します ---
    world.register::<components::Position>();
    world.register::<components::Velocity>();
    world.register::<components::Ball>();
    world.register::<components::Table>();

    // --- 4. シミュレーションの時間刻み dt をリソースとして World に登録します ---
    world.insert(TimeDelta {
        dt: Duration::from_secs_f32(config.dt),
    });

    // --- 5. エンティティ生成関数を用いて、各エンティティ（テーブル、手球、的球）を作成します ---
    // テーブル（ビリヤード台）エンティティを作成
    entities::create_table(&mut world, &config);
    // 手球（cue ball）エンティティを作成
    entities::create_cue_ball(&mut world, &config);
    // 的球（object ball）エンティティを作成
    entities::create_object_balls(&mut world, &config);

    // --- システムディスパッチャの構築 ---
    // システムの実行順序は、Physics → Collision → Print とします。
    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::PhysicsSystem, "physics_system", &[])
        .with(
            systems::CollisionSystem,
            "collision_system",
            &["physics_system"],
        )
        .with(
            systems::LoggingSystem,
            "print_system",
            &["collision_system"],
        )
        .build();

    // --- シミュレーションループ ---
    let steps = 10;
    for step in 0..steps {
        tracing::info!("--- Time step {} ---", step);
        // 各システムを順次実行します。
        dispatcher.dispatch(&world);
        // エンティティの生成／削除などの更新処理を実行します。
        world.maintain();
    }

    Ok(())
}
