# コンポーネントの結合（Joining Components）

前章では、`SystemData` を使用してリソースにアクセスする方法を学びました。  
コンポーネントにアクセスする場合は、単純に **`ReadStorage` をリクエストし、`Storage::get` で取得** できます。  
これは **単一のコンポーネントを取得する場合は問題ありません**。

しかし、次のような場合はどうでしょうか？

- **複数のコンポーネントを一括で処理したい**
- **一部のコンポーネントは必須だが、他のコンポーネントはオプション**
- **特定のコンポーネントを持たないエンティティを除外**

このような処理を **`Storage::get` だけで実装すると、非常に複雑で可読性が低くなります**。  
そこで、Specs では **"joining"（結合）** という仕組みを提供しています。

---

## 基本的な `join()`

前章で、**2 つのストレージを結合する基本的な例** を見ました。

```rust,ignore
for (pos, vel) in (&mut pos_storage, &vel_storage).join() {
    *pos += *vel;
}
```

このループは、**`pos_storage` と `vel_storage` の両方を持つエンティティ** について  
- **`pos_storage` のデータを更新**
- **`vel_storage` のデータを参照**

します。  
つまり、**すべての指定されたコンポーネントが必須になります**。

---

## エンティティ ID も取得する

エンティティの ID も取得したい場合は、**`&EntitiesRes` を追加** できます。

```rust,ignore
for (ent, pos, vel) in (&*entities, &mut pos_storage, &vel_storage).join() {
    println!("処理中のエンティティ: {:?}", ent);
    *pos += *vel;
}
```

取得したエンティティ ID を使えば、**後から特定のコンポーネントを手動で取得する** ことも可能です。

---

## オプションのコンポーネント

上記の例では、**すべてのエンティティに `Position` と `Velocity` が必要** でした。  
しかし、次のようなケースではどうすればよいでしょうか？

- `Mass`（質量）コンポーネントは持っている場合のみ処理したい
- **`Mass` を持っていないエンティティも処理対象に含めたい**

この場合、**`maybe()` を使用** します。

```rust,ignore
for (pos, vel, mass) in 
    (&mut pos_storage, &vel_storage, (&mut mass_storage).maybe()).join() {
    println!("処理中のエンティティ: {:?}", ent);
    *pos += *vel;

    if let Some(mass) = mass {
        let x = *vel / 300_000_000.0;
        let y = 1.0 - x * x;
        let y = y.sqrt();
        mass.current = mass.constant / y;
    }
}
```

### **このコードのポイント**
- **`maybe()` を使うと、コンポーネントが存在しない場合は `None` を返す**
- **`Mass` を持たないエンティティも `join()` の対象になる**
- **存在する場合のみ `Mass` を処理**

> **⚠️ 注意:**  
> `join()` に **`MaybeJoin` しか含まれていない** 場合、すべてのインデックスをループしてしまいます。  
> これを防ぐために、**`EntitiesRes` を追加してループ範囲をエンティティのみに制限** しましょう。

---

## `Storage::get()` を使った手動取得

`maybe()` を使うと効率的にエンティティを処理できますが、  
**特定のエンティティだけを対象にコンポーネントを取得したい場合** は、  
`Storage::get()` や `Storage::get_mut()` を使うこともできます。

```rust,ignore
for (target, damage) in (&target_storage, &damage_storage).join() {
    let target_health: Option<&mut Health> = health_storage.get_mut(target.ent);
    if let Some(target_health) = target_health {
        target_health.current -= damage.value;      
    }
}
```

このコードでは、**ダメージを受けたターゲットエンティティ** について

- `Health` がある場合のみ HP を減少させる
- `Health` がないエンティティは無視

する処理を行っています。

---

## 特定のコンポーネントを持たないエンティティを除外

特定のコンポーネントを **持っていないエンティティのみを処理** したい場合、  
**`!` 演算子を使ってフィルタリング** できます。

```rust,ignore
for (ent, pos, vel, ()) in (
    &*entities,
    &mut pos_storage,
    &vel_storage,
    !&frozen_storage, // ← `Frozen` を持っていないエンティティのみ
).join() {
    println!("処理中のエンティティ: {:?}", ent);
    *pos += *vel;
}
```

### **このループの条件**
- **`Position` を持っている**
- **`Velocity` を持っている**
- **`Frozen` を持っていない**

例えば、**`Frozen` コンポーネントがあるエンティティは移動できない** という設計にするときに便利です。

---

## `join()` の仕組み

**`join()` を呼び出せるのは `Join` トレイトを実装した型だけ** です。  
`join()` は **イテレータを返す** 仕組みになっています。

`Join` が実装されているのは、次のような型です。

- **`&ReadStorage<T>` / `&WriteStorage<T>`**（コンポーネントへの参照を返す）
- **`&mut WriteStorage<T>`**（コンポーネントへの可変参照を返す）
- **`&EntitiesRes`**（エンティティ ID を返す）
- **ビットセット（BitSet）**

ビットセットを活用することで、**さらに柔軟な結合** が可能になります。

---

## ビットセットを使った `join()`

Specs は [`hibitset`](https://github.com/slide-rs/hibitset) を使用しています。  
これにより、**レイヤードビットセット** を活用して効率的なコンポーネント管理が可能になります。

**カスタムビットセットを作成する例**

```rust,ignore
use hibitset::{BitSet, BitSetLike};

let mut bitset = BitSet::new();
bitset.add(entity1.id());
bitset.add(entity2.id());
```

**ビットセットの組み合わせ**
- **`&`（AND）** → 共通部分
- **`|`（OR）** → いずれかが `1` なら含める
- **`^`（XOR）** → どちらか一方のみ
- **`!`（NOT）** → 反転（例: 除外）

これを活用すると、**複雑なフィルタリングをシンプルに記述** できます。

---

## まとめ

- **`join()` を使うと、複数のコンポーネントを持つエンティティを簡単に処理できる**
- **`maybe()` を使うとオプションのコンポーネントを扱える**
- **`!` を使うと特定のコンポーネントを持たないエンティティを除外できる**
- **ビットセットを活用すると、より高度な結合が可能**

---

次の章では、**並列実行（Parallel Execution）** について学びます。