# `setup` ステージ（セットアップ段階）

これまで、コンポーネントのストレージやリソースを **`World` に手動で追加** してきました。  
しかし、**Specs では `setup` を使用すると、この作業を自動化できます**。

`setup` は **手動で呼び出すセットアップ処理** であり、`SystemData` を解析して  
- **コンポーネントを登録（register）**
- **リソースを挿入（insert）**

を自動で実行します（ただし、一部の例外を除く）。

`setup` 関数は次の場所で利用できます。

- **`ReadStorage` / `WriteStorage` / `Read` / `Write`**
- **`SystemData`**
- **`System`**
- **`RunNow`**
- **`Dispatcher`**
- **`ParSeq`**

### `setup` の挙動

- **すべてのコンポーネントが登録される**
- **`Default` を実装しているリソース、または `SetupHandler` を持つリソースが自動追加される**
- **`ReadExpect` / `WriteExpect` のリソースは自動追加されない**

通常、**`setup` は `Dispatcher` または `ParSeq` に対して実行** するのが推奨されます。  
これは、**システムグラフを構築した後、最初の `dispatch` を呼び出す前** に行います。  
この処理により、すべての `System` に対して `setup` が適用されます。

---

## `setup` の導入前後の比較

### **手動で `World` にコンポーネントとリソースを登録する方法**
```rust,ignore
use specs::prelude::*;

#[derive(Default)]
struct Gravity;

struct Velocity;

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}

struct SimulationSystem;

impl<'a> System<'a> for SimulationSystem {
    type SystemData = (Read<'a, Gravity>, WriteStorage<'a, Velocity>);

    fn run(&mut self, _: Self::SystemData) {}
}

fn main() {
    let mut world = World::new();
    world.insert(Gravity);
    world.register::<Velocity>();

    for _ in 0..5 {
        world.create_entity().with(Velocity).build();
    }

    let mut dispatcher = DispatcherBuilder::new()
        .with(SimulationSystem, "simulation", &[])
        .build();

    dispatcher.dispatch(&mut world);
    world.maintain();
}
```
---

### **`setup` を使用する方法**
```rust,ignore
fn main() {
    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new()
        .with(SimulationSystem, "simulation", &[])
        .build();

    dispatcher.setup(&mut world); // ← ここで自動セットアップ

    for _ in 0..5 {
        world.create_entity().with(Velocity).build();
    }

    dispatcher.dispatch(&mut world);
    world.maintain();
}
```

### **`setup` を使用するメリット**
- **コンポーネントやリソースの手動登録が不要になる**
- **すべての `SystemData` に対して自動的に `setup` が適用される**
- **コードがシンプルになり、エラーを減らせる**

---

## カスタム `setup` の実装

`setup` の便利な点は、単に `World` にデータを登録するだけでなく、  
**カスタムリソースの初期化や `System` のセットアップも可能** なことです。

次の例では、`shrev::EventChannel` を使用する `System` を作成します。

```rust,ignore
struct Sys {
    reader: ReaderId<Event>,
}

impl<'a> System<'a> for Sys {
    type SystemData = Read<'a, EventChannel<Event>>;

    fn run(&mut self, events: Self::SystemData) {
        for event in events.read(&mut self.reader) {
            [..]
        }
    }
}
```

このコードには **問題点** があります。

- `Sys` は `ReaderId<Event>` を必要とする
- **`ReaderId` を取得するには `EventChannel<Event>` がすでに `World` に存在している必要がある**
- ユーザーが手動で `EventChannel` を `World` に追加しないと動作しない

### **解決策：カスタム `setup` を実装**
```rust,ignore
use specs::prelude::*;

#[derive(Default)]
struct Sys {
    reader: Option<ReaderId<Event>>,
}

impl<'a> System<'a> for Sys {
    type SystemData = Read<'a, EventChannel<Event>>;

    fn run(&mut self, events: Self::SystemData) {
        for event in events.read(&mut self.reader.as_mut().unwrap()) {
            [..]
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world); // ← 重要：デフォルトの `setup` を呼び出す
        self.reader = Some(world.fetch_mut::<EventChannel<Event>>().register_reader());
    }
}
```

### **この方法のメリット**
- **ユーザーが `EventChannel` を `World` に手動で登録する必要がなくなる**
- **カスタム `setup` により `Sys` の初期化を完全に自動化**
- **エラーや設定漏れを防ぐ**

> **注意:**  
> **`setup` をオーバーライドする場合は `Self::SystemData::setup(world);` を必ず呼び出すこと！**  
> これを忘れると、システムの `SystemData` が適切にセットアップされず、  
> **最初の `dispatch` で panic する可能性があります。**

---

## 一括セットアップ（Bulk Setup）

`specs` を利用するライブラリでは、  
**複数のコンポーネントやリソースを一括でセットアップする方法** を提供すると便利です。

一般的な方法としては、**個別のシステムをユーザーに登録させる** 形を推奨します。

```rust,ignore
fn add_physics_engine(world: &mut World, config: LibraryConfig) -> Result<(), LibraryError> {
    world.register::<Velocity>();
    // 他のコンポーネントやリソースも登録
}
```

---

## まとめ

- **`setup` を使用すると、コンポーネントやリソースの登録を自動化できる**
- **`setup` は `Dispatcher` または `ParSeq` に対して実行するのが推奨**
- **カスタム `setup` を実装することで、リソースの初期化を自動化可能**
- **`setup` をオーバーライドする場合は `Self::SystemData::setup(world);` を必ず呼ぶ**
- **ライブラリでは一括セットアップ関数を提供すると便利**

---

次の章では、**並列実行の詳細（Parallel Execution）** について説明します。