// src/config.rs
//
// このファイルでは、YAML から読み込む設定情報の構造体と、
// 設定ファイルを読み込む関数 load_config を定義しています。

use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

/// シミュレーションに必要な各種設定情報を保持する構造体です。
#[derive(Debug, Deserialize)]
pub struct Config {
    // シミュレーションの時間刻み（秒）
    pub dt: f32,
    // ビリヤード台の寸法情報
    pub table: TableConfig,
    // 手球、的球共通の物理パラメータ
    pub ball: BallConfig,
    // 手球（cue ball）の初期位置・初速度情報
    pub cue_ball: CueBallConfig,
    // 的球の配置情報
    pub object_balls: ObjectBallsConfig,
}

/// テーブルの寸法情報を保持する構造体です。
#[derive(Debug, Deserialize)]
pub struct TableConfig {
    pub width: f32,
    pub height: f32,
}

/// ボールの物理特性を保持する構造体です。（手球、的球共通）
#[derive(Debug, Deserialize)]
pub struct BallConfig {
    pub radius: f32,
    pub mass: f32,
    pub restitution: f32,
}

/// 手球の初期位置・初速度情報を保持する構造体です。
#[derive(Debug, Deserialize)]
pub struct CueBallConfig {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

/// 的球の配置情報を保持する構造体です。
#[derive(Debug, Deserialize)]
pub struct ObjectBallsConfig {
    pub positions: Vec<PositionConfig>,
}

/// 各ボールの初期位置情報を保持する構造体です。
#[derive(Debug, Deserialize)]
pub struct PositionConfig {
    pub x: f32,
    pub y: f32,
}

/// 指定されたパスから YAML 設定ファイルを読み込み、Config を返す関数です。
///
/// # 引数
/// - `path`: 設定ファイルのパス
///
/// # 戻り値
/// 読み込みに成功した場合は Config、失敗した場合は Error を返します。
pub fn load_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config: Config = serde_yaml::from_reader(reader)?;
    Ok(config)
}
