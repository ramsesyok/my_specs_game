# Hello, `World`!

## セットアップ

まずは、`specs` を試していただきありがとうございます！  
プロジェクトのセットアップを行う前に、**最新の Rust バージョンを使用していることを確認** してください。

```bash
rustup update
```

では、プロジェクトをセットアップしましょう！

```bash
cargo new --bin my_game
```

次に、以下の行を `Cargo.toml` に追加してください。

```toml
[dependencies]
specs = "0.16.1"
```

---

## コンポーネントの作成

まずは、データを定義してみましょう。

```rust,ignore
use specs::{Component, VecStorage};

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct Velocity {
    x: f32,
    y: f32,
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}
```

これは `Position`（位置）と `Velocity`（速度）の **2 つのコンポーネント** です。  
また、`specs-derive` クレートを使用すると、より簡潔にコンポーネントを定義できます。

まず、`derive` 機能を有効にする必要があります。

```toml
[dependencies]
specs = { version = "0.16.1", features = ["specs-derive"] }
```

これにより、次のように記述できるようになります。

```rust,ignore
use specs::{Component, VecStorage};

#[derive(Component, Debug)]
#[storage(VecStorage)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
struct Velocity {
    x: f32,
    y: f32,
}
```

`#[storage(...)]` を省略すると、デフォルトでは `DenseVecStorage` が使用されます。  
ただし、この例では `VecStorage` を明示的に指定しています（詳細は [ストレージの章][sc] を参照）。

また、以下のように `FlaggedStorage` などの **より複雑なストレージ指定** も可能です。

```rust,ignore
#[derive(Component, Debug)]
#[storage(FlaggedStorage<Self, DenseVecStorage<Self>>)]
pub struct Data {
    [..]
}
```

（詳細は [`FlaggedStorage` と変更イベントの章][tc] を参照）

---

## `World` の作成

次に、コンポーネントを格納するための **`World`** を作成します。

```rust,ignore
use specs::{World, WorldExt, Builder};

let mut world = World::new();
world.register::<Position>();
world.register::<Velocity>();
```

これにより、`Position` と `Velocity` のコンポーネントストレージが作成されます。

エンティティ（ゲーム内オブジェクト）を作成し、`Position` を関連付けてみましょう。

```rust,ignore
let ball = world.create_entity().with(Position { x: 4.0, y: 7.0 }).build();
```

これで、**エンティティが `Position` を持つ状態** になりました。

> **補足:** `World` は `shred` というライブラリの一部であり、Specs の重要な依存関係です。  
> `WorldExt` トレイトをインポートすることで、Specs の関数を使用できます。

しかし、まだデータを保存しただけで、**何も処理をしていません**。  
次は、**システムを作成** して、データを活用してみましょう。

---

## システムの作成

```rust,ignore
use specs::System;

struct HelloWorld;

impl<'a> System<'a> for HelloWorld {
    type SystemData = ();

    fn run(&mut self, data: Self::SystemData) {}
}
```

これは **最も基本的なシステム** ですが、まだ何もしません。  
このシステムは `SystemData` という関連型を持ち、**システムが使用するコンポーネントの種類** を定義します。

では、`Position` コンポーネントを読み取るようにしてみましょう。

```rust,ignore
use specs::{ReadStorage, System};

struct HelloWorld;

impl<'a> System<'a> for HelloWorld {
    type SystemData = ReadStorage<'a, Position>;

    fn run(&mut self, position: Self::SystemData) {
        use specs::Join;

        for position in position.join() {
            println!("Hello, {:?}", &position);
        }
    }
}
```

ここでのポイント：

- `ReadStorage<'a, Position>` を `SystemData` に指定することで、**`Position` コンポーネントを読み取るシステム** になる。
- `.join()` を使用して、すべての `Position` をループ処理する。

> **注意:** システムで使用するコンポーネントは、`world.register::<Component>()` を **事前に登録** しておく必要があります。  
> そうしないと、システム実行時に **パニック（実行時エラー）** になります。  
> 通常は `setup` の段階で自動登録されますが、詳細は後の [セットアップの章][se] で説明します。

> システムデータとして使用できる型は他にもたくさんあります。  
> 詳細は [システムデータの章][cs] を参照してください。

---

## システムの実行

このシステムは、すべての `Position` コンポーネントを取得し、表示するだけです。  
実行するには `RunNow` を使用します。

```rust,ignore
use specs::RunNow;

let mut hello_world = HelloWorld;
hello_world.run_now(&world);
world.maintain();
```

> `world.maintain()` は **必須ではありません** が、  
> **エンティティの作成や削除が発生する場合には呼び出す必要があります**。  
> これは、エンティティの変更を内部データ構造に適用するために必要です。

---

## 完全なコード例

ここまでのコードをまとめると、次のようになります。

```rust,ignore
use specs::{Builder, Component, ReadStorage, System, VecStorage, World, WorldExt, RunNow};

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct Velocity {
    x: f32,
    y: f32,
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}

struct HelloWorld;

impl<'a> System<'a> for HelloWorld {
    type SystemData = ReadStorage<'a, Position>;

    fn run(&mut self, position: Self::SystemData) {
        use specs::Join;

        for position in position.join() {
            println!("Hello, {:?}", &position);
        }
    }
}

fn main() {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();

    world.create_entity().with(Position { x: 4.0, y: 7.0 }).build();

    let mut hello_world = HelloWorld;
    hello_world.run_now(&world);
    world.maintain();
}
```

---

ここまでは **基本的な例** でした。  
しかし、ECS の**真の強み**である **Dispatcher（並列処理）** をまだ扱っていません。

次は [第 3 章: Dispatcher][c3] で、**並列実行** について学びましょう。

[c3]: ./03_dispatcher.html