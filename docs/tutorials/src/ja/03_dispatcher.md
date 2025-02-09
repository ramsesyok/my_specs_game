# Dispatcher（ディスパッチャー）

## `Dispatcher` を使うべき場面

`Dispatcher` を使用すると、可能な限り **システムの並列実行** を自動化できます。  
これは [フォーク・ジョインモデル][fj] を使用して処理を分割し、最終的に結果を統合します。  
多少のオーバーヘッドが発生する可能性がありますが、大規模なゲームを構築する場合には非常に便利です。  
手動で並列処理を管理しなくてもよくなります。

[fj]: https://en.wikipedia.org/wiki/Fork–join_model

---

## `Dispatcher` の構築

まず、`Dispatcher` を作成してみましょう。

```rust,ignore
use specs::DispatcherBuilder;

let mut dispatcher = DispatcherBuilder::new()
    .with(HelloWorld, "hello_world", &[])
    .build();
```

このコードの意味を見てみましょう。

1. `DispatcherBuilder::new()` でディスパッチャーを作成
2. `.with(HelloWorld, "hello_world", &[])` を追加
   - `HelloWorld` という **システム** を登録
   - `"hello_world"` という **名前** を付ける
   - 依存関係は **なし** (`&[]`)
3. `.build()` で **ビルド** して `dispatcher` を作成

この **名前** は、別のシステムの依存関係を指定する際に使用できます。  
しかし、今のところ **他のシステムはまだ存在しません**。

---

## 別のシステムを追加

次に、新しいシステム `UpdatePos` を作成してみましょう。

```rust,ignore
struct UpdatePos;

impl<'a> System<'a> for UpdatePos {
    type SystemData = (ReadStorage<'a, Velocity>,
                       WriteStorage<'a, Position>);
}
```

`SystemData` を見てみると、**タプル** を使用して 2 つのコンポーネントを指定しています。

- `ReadStorage<'a, Velocity>` → **読み取り専用の `Velocity`**
- `WriteStorage<'a, Position>` → **書き込み可能な `Position`**

実は、**`SystemData` は最大 26 個の要素を持つタプルを使用可能** です。

> **補足:** `ReadStorage` と `WriteStorage` は、それ自体が `SystemData` を実装しているため、  
> `HelloWorld` システムでは `ReadStorage` を直接 `SystemData` として指定できました。  
> 詳細は [システムデータの章][cs] を参照してください。

[cs]: ./06_system_data.html

### `run` メソッドの実装

```rust,ignore
fn run(&mut self, (vel, mut pos): Self::SystemData) {
    use specs::Join;
    for (vel, pos) in (&vel, &mut pos).join() {
        pos.x += vel.x * 0.05;
        pos.y += vel.y * 0.05;
    }
}
```

この `.join()` は、2 つの **コンポーネントストレージを結合** します。

- `Position` **だけ** を持つエンティティ
- `Velocity` **だけ** を持つエンティティ
- **どちらも持たない** エンティティ  
  → **スキップされる**

また、`0.05` は **デルタタイム（フレーム間の時間）** の代用としてハードコードしています。  
本来は `Resource` を使うべきですが、詳細は次の [リソースの章][c4] で解説します。

[c4]: ./04_resources.html

---

## システムの依存関係を追加

ここで、新しいシステムを **既存のシステムに依存** させてみます。

```rust,ignore
    .with(UpdatePos, "update_pos", &["hello_world"])
    .with(HelloWorld, "hello_updated", &["update_pos"])
```

- **`UpdatePos`** は `"hello_world"` に依存
  → `HelloWorld` **の実行が完了した後** に実行される。
- **`HelloWorld`**（`"hello_updated"`）は `"update_pos"` に依存
  → **`UpdatePos` の後に実行される。**

このように **明示的に実行順序を指定** できます。

---

## `Dispatcher` の実行

システムをすべて実行するには、次のように `dispatch` を呼び出します。

```rust,ignore
dispatcher.dispatch(&mut world);
```

---

## 完全なコード例

これまでのコードを **1 つのプログラム** にまとめました。

```rust,ignore
use specs::{Builder, Component, DispatcherBuilder, ReadStorage,
            System, VecStorage, World, WorldExt, WriteStorage};

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

struct UpdatePos;

impl<'a> System<'a> for UpdatePos {
    type SystemData = (ReadStorage<'a, Velocity>,
                       WriteStorage<'a, Position>);

    fn run(&mut self, (vel, mut pos): Self::SystemData) {
        use specs::Join;
        for (vel, pos) in (&vel, &mut pos).join() {
            pos.x += vel.x * 0.05;
            pos.y += vel.y * 0.05;
        }
    }
}

fn main() {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();

    // 最初のエンティティは `Velocity` を持たないため、位置は変わらない
    world.create_entity().with(Position { x: 4.0, y: 7.0 }).build();

    // 2 つ目のエンティティは `Velocity` を持つため、位置が更新される
    world
        .create_entity()
        .with(Position { x: 2.0, y: 5.0 })
        .with(Velocity { x: 0.1, y: 0.2 })
        .build();

    let mut dispatcher = DispatcherBuilder::new()
        .with(HelloWorld, "hello_world", &[])
        .with(UpdatePos, "update_pos", &["hello_world"])
        .with(HelloWorld, "hello_updated", &["update_pos"])
        .build();

    dispatcher.dispatch(&mut world);
    world.maintain();
}
```

---

## まとめ

- **`Dispatcher` を使うことで、システムを並列実行できる。**
- **`SystemData` を使用して、システムが必要なコンポーネントを定義できる。**
- **`.join()` を使うと、関連するコンポーネントを持つエンティティだけを処理できる。**
- **`dispatcher.dispatch(&mut world)` で、すべてのシステムを実行できる。**
- **依存関係を指定することで、実行順序を制御できる。**

---

次の [リソースの章][c4] では、**システム間で共有するデータ**（`Resource`）について解説します。