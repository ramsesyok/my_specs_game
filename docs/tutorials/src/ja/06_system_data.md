# システムデータ（System Data）

各システムは **実行に必要なデータ** をリクエストできます。  
これは `System::SystemData` 型を使用して指定します。

`SystemData` トレイトを実装する代表的な型は次のとおりです。

- **`ReadStorage<T>`**：コンポーネント `T` の**読み取り**
- **`WriteStorage<T>`**：コンポーネント `T` の**書き込み**
- **`Read<T>`**：リソース `T` の**読み取り**
- **`Write<T>`**：リソース `T` の**書き込み**
- **`ReadExpect<T>` / `WriteExpect<T>`**：リソースが存在しない場合に **panic する `Read` / `Write`**
- **`Entities`**：エンティティの作成・削除を扱う

また、**`SystemData` はタプルに対して自動的に実装される** ため、  
複数の `SystemData` を組み合わせることも可能です。

例えば、次のようにシステムを定義できます。

```rust
struct Sys;

impl<'a> System<'a> for Sys {
    type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>);

    fn run(&mut self, (pos, vel): Self::SystemData) {
        /* ... */
    }
}
```

---

## `ReadStorage` / `WriteStorage` の注意点

**同じコンポーネントに対して `ReadStorage` と `WriteStorage` を同時に使用することはできません。**  
また、同じリソースに対して `Read` と `Write` を同時に使用することもできません。

これは **Rust の借用ルール** と同じであり、**Specs は実行時にこれをチェック** します。  
違反すると **panic** になるので注意してください。

---

## エンティティの作成・削除

システム内でエンティティを作成・削除する場合は、**`Entities` を使用** します。  
これは `SystemData` を実装しているため、タプルに追加するだけで利用できます。

> **注意:**  
> **`specs::Entities`** と **`specs::EntitiesRes`** は異なります。  
> - **`EntitiesRes`** は **リソース**
> - **`Entities`** は **`Read<Entities>` の型エイリアス**

### エンティティの削除

エンティティを一定時間後に削除する `DecaySys` の例を見てみましょう。

```rust
pub struct Life {
    life: f32,
}

struct DecaySys;

impl<'a> System<'a> for DecaySys {
    type SystemData = (Entities<'a>, WriteStorage<'a, Life>);

    fn run(&mut self, (entities, mut life): Self::SystemData) {
        for (e, life) in (&entities, &mut life).join() {
            if life.life < 0.0 {
                entities.delete(e);
            } else {
                life.life -= 1.0;
            }
        }
    }
}
```

> **注意:**  
> **エンティティを削除した後は、`World::maintain()` を呼び出す必要があります。**  
> これにより、関連するコンポーネントが適切に削除されます。

---

## コンポーネントの追加・削除

コンポーネントを追加・削除する方法は 2 つあります。

1. **直接 `WriteStorage` を操作する**
2. **`LazyUpdate` を使用する**

```rust,ignore
use specs::{Component, Read, LazyUpdate, NullStorage, System, Entities, WriteStorage};

struct Stone;
impl Component for Stone {
    type Storage = NullStorage<Self>;
}

struct StoneCreator;
impl<'a> System<'a> for StoneCreator {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Stone>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (entities, mut stones, updater): Self::SystemData) {
        let stone = entities.create();

        // 1) 直接 `WriteStorage` を使って追加
        stones.insert(stone, Stone);

        // 2) `LazyUpdate` を使って遅延的に追加
        updater.insert(stone, Stone);
    }
}
```

> **注意:**  
> `LazyUpdate` を使用した場合、**`World::maintain()` を呼び出す必要があります。**  
> これにより、遅延的な変更が適用されます。

---

## `SetupHandler` / `Default` を持たないリソースの扱い

Specs では、リソースを自動作成するために **`Default` を実装することが推奨** されています。  
ただし、`Default` を実装できないリソースも存在します。

その場合、次のいずれかの方法を使用できます。

1. **`SetupHandler` を実装する**
2. **`ReadExpect<T>` / `WriteExpect<T>` を使用する**
   - `Read<T>` や `Write<T>` とは異なり、リソースが `World` に存在しないと **panic する**
3. **`Option<Read<T>>` を使用する**
   - **リソースが存在しなくてもエラーにならない**

詳細は [リソースの章][c4] を参照してください。

[c4]: ./04_resources.html

---

## `SystemData` の構造体によるカスタマイズ

これまでの例では **タプル** を使って `SystemData` を定義してきましたが、  
**要素が多くなると管理が難しくなります**。

その場合、**構造体を使って `SystemData` をまとめる** 方法があります。

```rust,ignore
extern crate specs;

use specs::prelude::*;
// `shred` を `SystemData` の derive に必要
use specs::shred;

#[derive(SystemData)]
pub struct MySystemData<'a> {
    positions: ReadStorage<'a, Position>,
    velocities: ReadStorage<'a, Velocity>,
    forces: ReadStorage<'a, Force>,

    delta: Read<'a, DeltaTime>,
    game_state: Write<'a, GameState>,
}
```

この方法を使うと、**可読性が向上し、コードの管理が容易になります**。

> **注意:**  
> `SystemData` を構造体として定義するには、  
> **`shred-derive` 機能を `Cargo.toml` で有効にする必要があります。**

```toml
specs = { version = "*", features = ["shred-derive"] }
```

---

## まとめ

- **`SystemData` を使うと、システム内でコンポーネントやリソースを取得できる**
- **`ReadStorage<T>` / `WriteStorage<T>` を使ってコンポーネントを操作**
- **`Read<T>` / `Write<T>` を使ってリソースを操作**
- **`Entities` を使ってエンティティを作成・削除**
- **`LazyUpdate` を使うと遅延的な変更が可能**
- **タプルの代わりに構造体を使うと管理が容易になる**

---

次の章では、**セットアップ（Setup）** について説明します。