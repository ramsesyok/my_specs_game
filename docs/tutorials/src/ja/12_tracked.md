# `FlaggedStorage` と変更イベント（Modification Events）

ほとんどのゲームでは **多くのエンティティが存在** しますが、  
**毎フレーム、すべてのコンポーネントが変更されるわけではありません**。

例えば：
- **変更があったコンポーネントのみを更新したい**
- **外部リソース（グラフィックスや物理エンジン）とデータを同期したい**
- **不要な計算を減らして、パフォーマンスを最適化したい**

このような場面で役立つのが **`FlaggedStorage`** です。  
`FlaggedStorage` を使うと、**変更イベントを検知し、変更されたエンティティのみを処理** できます。

---

## `FlaggedStorage` の仕組み

**通常の `Storage` を `FlaggedStorage` でラップ** すると：
1. **コンポーネントの変更がトラッキングされる**
2. **変更があったエンティティだけを特定できる**
3. **変更イベントを `BitSet` に格納し、効率的に処理可能**

次のコードを見てみましょう。

```rust,ignore
pub struct Data {
    [..]
}

impl Component for Data {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

#[derive(Default)]
pub struct Sys {
    pub dirty: BitSet, // 変更があったエンティティのビットセット
    pub reader_id: Option<ReaderId<ComponentEvent>>,
}

impl<'a> System<'a> for Sys {
    type SystemData = (
        ReadStorage<'a, Data>,
        WriteStorage<'a, SomeOtherData>,
    );

    fn run(&mut self, (data, mut some_other_data): Self::SystemData) {
        self.dirty.clear();

        let events = data.channel().read(self.reader_id.as_mut().unwrap());

        for event in events {
            match event {
                ComponentEvent::Modified(id) | ComponentEvent::Inserted(id) => {
                    self.dirty.add(*id);
                }
                // `Removed` イベントは処理不要（`Join` で自動的にフィルタリングされる）
                ComponentEvent::Removed(_) => (),
            }
        }

        for (d, other, _) in (&data, &mut some_other_data, &self.dirty).join() {
            // `d` の変更を `other` に反映
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.reader_id = Some(
            WriteStorage::<Data>::fetch(&res).register_reader()
        );
    }
}
```

### **コードのポイント**
- **`FlaggedStorage<Self, DenseVecStorage<Self>>` を指定**
  - `Data` コンポーネントのストレージを `FlaggedStorage` に変更
- **変更イベントを `BitSet` に格納**
  - `ComponentEvent::Modified(id)` → コンポーネントが変更された
  - `ComponentEvent::Inserted(id)` → 新しく追加された
  - `ComponentEvent::Removed(id)` → 削除された（通常は処理不要）

> **`join()` の対象に `&self.dirty` を追加すると、変更があったエンティティだけを処理可能！**

---

## `FlaggedStorage` の変更イベント

`FlaggedStorage` は **次の 3 つのイベントを発行** します。

1. **`ComponentEvent::Inserted`**  
   → **コンポーネントが新しく追加されたとき**

2. **`ComponentEvent::Modified`**  
   → **コンポーネントが変更されたとき**

3. **`ComponentEvent::Removed`**  
   → **コンポーネントが削除されたとき**

通常、`Removed` は特別な処理をしなくても `Join` で自動的に除外されるため、  
多くのケースでは `Inserted` / `Modified` のみを処理すれば十分です。

---

## `FlaggedStorage` の注意点

### **⚠️ `join()` で `mut` にすると、すべてのコンポーネントが変更扱いになる**
次のコードは **変更がなくても `Modified` としてフラグが立つ** ため、**絶対に避けるべき** です。

```rust,ignore
// ⚠️ `FlaggedStorage` を使用している場合、このコードは NG！
//
// `join()` で `mut` を取得するだけで、すべてのコンポーネントが "変更された" とマークされる。
for comp in (&mut comps).join() {
    // 何もしなくても変更扱いになってしまう
}
```

### **✅ 正しい方法**
**`BitSet` で対象を絞る or `RestrictedStorage` を使う**
```rust,ignore
for (comp, _) in (&mut comps, &self.dirty).join() {
    // `dirty` に含まれるエンティティのみ処理
}
```

または **`RestrictedStorage` を使用** し、  
**変更が必要な場合のみ `mut` で取得** する。

```rust,ignore
for (entity, mut comp) in (&entities, &mut comps.restrict_mut()).join() {
    // 条件を満たす場合のみ `mut` で取得
    if comp.get().condition < 5 {
        let mut comp = comp.get_mut();
        // 変更処理
    }
}
```

---

## `storage.set_event_emission(false)`

場合によっては、**一時的に変更イベントを無効化** したいこともあります。  
例えば、**内部的な更新処理の間は `Modified` を発行したくない** ケースです。

その場合は、`set_event_emission(false)` を使います。

```rust,ignore
storage.set_event_emission(false); // イベント発行を無効化

// 変更処理
for comp in (&mut comps).join() {
    // ここで変更しても `Modified` は発行されない
}

storage.set_event_emission(true); // イベント発行を再開
```

---

## まとめ

- **`FlaggedStorage` を使うと、変更があったエンティティのみを処理可能**
- **`ComponentEvent` を使って `Inserted` / `Modified` / `Removed` を検知**
- **`join()` で `mut` を取ると、全コンポーネントが変更扱いになるので注意**
  - ✅ `BitSet` や `RestrictedStorage` を使って対象を絞る
- **`set_event_emission(false)` を使うと、特定の処理中は変更イベントを無効化できる**

---

次の章では、**外部システムとのデータ同期（External Synchronization）** について学びます。