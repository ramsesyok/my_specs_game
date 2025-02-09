# リソース（Resources）

この（短い）章では、**リソース（Resource）** の概念について説明します。  
リソースとは、**システム間で共有されるデータ** のことです。

まず、リソースが必要になるのはどのような場面でしょうか？  
実は、[第 3 章][c3] の例が良い例です。  
そこで私たちは **デルタタイム（フレーム間の時間）を仮の値として扱いました**。  
では、正しくリソースを使う方法を見てみましょう。

[c3]: ./03_dispatcher.html

---

## リソースの定義

まず、**デルタタイムをリソースとして定義** してみます。

```rust,ignore
#[derive(Default)]
struct DeltaTime(f32);
```

> **注意:** 実際のゲームでは `std::time::Duration` を使用することを推奨します。  
> `f32` は **時間を扱うには精度が不足している** ため、誤差が発生しやすくなります。

---

## リソースの追加と更新

このリソースを `World` に追加するのは簡単です。

```rust,ignore
world.insert(DeltaTime(0.05)); // 初期値を設定
```

デルタタイムの値を更新するには、次のようにします。

```rust,ignore
use specs::WorldExt;

let mut delta = world.write_resource::<DeltaTime>();
*delta = DeltaTime(0.04);
```

---

## システムからリソースを参照する

予想できるかもしれませんが、リソースを扱うための `SystemData` が存在します。  
それが `Read`（読み取り専用）と `Write`（書き込み可能）です。

では、システムをリソースを使う形に書き換えてみましょう。

```rust,ignore
use specs::{Read, ReadStorage, System, WriteStorage};

struct UpdatePos;

impl<'a> System<'a> for UpdatePos {
    type SystemData = (Read<'a, DeltaTime>,  // DeltaTime を読み取る
                       ReadStorage<'a, Velocity>,
                       WriteStorage<'a, Position>);

    fn run(&mut self, data: Self::SystemData) {
        let (delta, vel, mut pos) = data;

        // `Read` は `Deref` を実装しているので `&DeltaTime` に変換できる
        let delta = delta.0;

        for (vel, pos) in (&vel, &mut pos).join() {
            pos.x += vel.x * delta;
            pos.y += vel.y * delta;
        }
    }
}
```

このコードでは、**デルタタイムを `Read<'a, DeltaTime>` としてシステムに渡す** ことで、  
`0.05` のようなハードコーディングを **リソースからの取得** に置き換えています。

> **注意:** システムがアクセスするリソースは、**事前に `world.insert(resource)` で登録** する必要があります。  
> そうしないと **パニック（実行時エラー）** になります。  
> ただし、リソースに `Default` 実装がある場合は `setup` の段階で自動登録されるため、  
> 通常は明示的に登録しなくても動作します（詳細は後の章で説明）。

> `SystemData` の詳細については [システムデータの章][cs] を参照してください。

[cs]: ./06_system_data.html

---

## `Default` を持たないリソースの扱い

これまでの例では、リソースに `#[derive(Default)]` を付けていました。  
しかし、**`Default` を実装できないリソース** を扱う場合はどうすればよいでしょうか？

`Read` や `Write` はデフォルトで `Default` を必要とします。  
これは、Specs が `setup` 時に **デフォルト値を自動で `World` に追加しようとする** ためです。

`Default` を実装できないリソースを扱う方法は 3 つあります。

1. **`SetupHandler` をカスタム実装する**
   - `SystemData` で `Read<'a, Resource, TheSetupHandlerType>` を指定
2. **`Read` / `Write` の代わりに `ReadExpect` / `WriteExpect` を使う**
   - これを使うと、**リソースが `World` に登録されていなければ `panic` する**
3. **`Option<Read<'a, Resource>>` を使う**
   - リソースが **本当にオプション** の場合に使用
   - ただし、**`Read<'a, Option<Resource>>` とは異なる**（後者は `World` に `Option<Resource>` が存在する前提になる）

---

## まとめ

- **リソースはシステム間で共有されるデータ。**
- **`Read<T>` を使うとリソースを読み取れる。**
- **`Write<T>` を使うとリソースを更新できる。**
- **リソースは `world.insert(resource)` で登録する必要がある。**
- **`Default` を持たないリソースは `ReadExpect` や `Option<Read<T>>` を使って対処できる。**

---

次の [ストレージの章][c5] では、さまざまな **ストレージの種類** と **用途に応じた選び方** について解説します。

[c5]: ./05_storages.html