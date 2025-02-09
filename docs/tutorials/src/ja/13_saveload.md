# `Saveload`（保存と読み込み）

`Saveload` は **Specs の `World` をシリアライズ（保存）およびデシリアライズ（読み込み）するためのモジュール** です。  
この機能は **[`serde`]** を使用しており、`Cargo.toml` で `serde` 機能を有効にする必要があります。

[`serde`]: https://docs.rs/serde

**仕組みの概要**
1. **`Marker` コンポーネント** を定義する
2. **`MarkerAllocator` リソース** を作成する
3. **マーカーが付いたエンティティのみがシリアライズ / デシリアライズ対象になる**
4. **`SerializeComponents` / `DeserializeComponents` がデータの入出力を行う**

---

## `Marker` と `MarkerAllocator`

`Marker` と `MarkerAllocator<M: Marker>` は **トレイト** であり、  
シンプルな実装として **`SimpleMarker<T>`** と **`SimpleMarkerAllocator<T>`** が利用できます。  
これらは **Zero-Sized Type（ZST）** であり、メモリを消費しません。

### **基本的な `Marker` の使用例**
```rust,ignore
struct NetworkSync;
struct FilePersistent;

fn main() {
    let mut world = World::new();

    // `NetworkSync` 用のマーカーとアロケータを登録
    world.register::<SimpleMarker<NetworkSync>>();
    world.insert(SimpleMarkerAllocator::<NetworkSync>::default());

    // `FilePersistent` 用のマーカーとアロケータを登録
    world.register::<SimpleMarker<FilePersistent>>();
    world.insert(SimpleMarkerAllocator::<FilePersistent>::default());

    // エンティティを作成し、両方のマーカーを付与
    world
        .create_entity()
        .marked::<SimpleMarker<NetworkSync>>()
        .marked::<SimpleMarker<FilePersistent>>()
        .build();
}
```

---

## カスタム `Marker` の実装

独自の `Marker` を作成することもできます。

### **カスタム `Marker` の実装例**
```rust,ignore
use specs::{prelude::*, saveload::{MarkedBuilder, Marker, MarkerAllocator}};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
struct MyMarker(u64);

impl Component for MyMarker {
    type Storage = VecStorage<Self>;
}

impl Marker for MyMarker {
    type Identifier = u64;
    type Allocator = MyMarkerAllocator;

    fn id(&self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct MyMarkerAllocator(std::collections::HashMap<u64, Entity>);

impl MarkerAllocator<MyMarker> for MyMarkerAllocator {
    fn allocate(&mut self, entity: Entity, id: Option<u64>) -> MyMarker {
        let id = id.unwrap_or_else(|| self.unused_key());
        self.0.insert(id, entity);
        MyMarker(id)
    }

    fn retrieve_entity_internal(&self, id: u64) -> Option<Entity> {
        self.0.get(&id).cloned()
    }

    fn maintain(
        &mut self,
        entities: &EntitiesRes,
        storage: &ReadStorage<MyMarker>,
    ) {
        self.0 = (entities, storage)
            .join()
            .map(|(entity, marker)| (marker.0, entity))
            .collect();
    }
}

fn main() {
    let mut world = World::new();

    world.register::<MyMarker>();
    world.insert(MyMarkerAllocator::default());

    world
        .create_entity()
        .marked::<MyMarker>()
        .build();
}
```

### **ポイント**
- **`Marker` はエンティティに一意の ID を割り当てる**
- **`MarkerAllocator` は ID を管理し、エンティティを検索可能にする**
- **マーカーの付与には `MarkedBuilder` トレイトを使用**

---

## 既存エンティティに `Marker` を追加

既存のエンティティにマーカーを追加するには、  
`MarkerAllocator::mark()` を使用します。

```rust,ignore
fn mark_entity(
    entity: Entity,
    mut allocator: Write<SimpleMarkerAllocator<A>>,
    mut storage: WriteStorage<SimpleMarker<A>>,
) {
    use MarkerAllocator; // `mark()` メソッドを使用

    match allocator.mark(entity, &mut storage) {
        None => println!("エンティティは既に削除されていた"),
        Some((_, false)) => println!("エンティティは既にマークされている"),
        Some((_, true)) => println!("エンティティを正常にマーク"),
    }
}
```

---

## シリアライズとデシリアライズ

Specs では、データの入出力に **`SerializeComponents` / `DeserializeComponents`** を使用します。

### **シリアライズ**
```rust,ignore
specs::saveload::SerializeComponents
    ::<Infallible, SimpleMarker<A>>
    ::serialize(
        &(position_storage, mass_storage),  // シリアライズ対象の ReadStorage タプル
        &entities,                          // Entities<'a>
        &marker_storage,                    // マーカー ReadStorage
        &mut serializer,                     // `serde::Serializer`
    )   // Result<Serializer::Ok, Serializer::Error>
```

### **デシリアライズ**
```rust,ignore
specs::saveload::DeserializeComponents
    ::<Infallible, SimpleMarker<A>>
    ::deserialize(
        &mut (position_storage, mass_storage),  // デシリアライズ対象の WriteStorage タプル
        &entities,                              // Entities<'a>
        &mut marker_storage,                    // マーカー WriteStorage
        &mut marker_allocator,                  // `MarkerAllocator`
        &mut deserializer,                       // `serde::Deserializer`
    )   // Result<(), Deserializer::Error>
```

---

## `World` から `SystemData` を取得する方法

`SystemData` を取得するには、システム内で取得するか、  
`World::system_data()` を使用します。

```rust,ignore
let (
    entities,
    mut marker_storage,
    mut marker_allocator,
    mut position_storage,
    mut mass_storage,
) = world.system_data::<(
    Entities,
    WriteStorage<SimpleMarker<A>>,
    Write<SimpleMarkerAllocator<A>>,
    WriteStorage<Position>,
    WriteStorage<Mass>,
)>();
```

---

## `ConvertSaveload` の実装

シリアライズ / デシリアライズする `Component` は **`ConvertSaveload` を実装** する必要があります。

- **`Clone + serde::Serialize + serde::DeserializeOwned` を満たすと自動実装**
- **カスタム型の場合は `specs-derive` で自動導出可能**

```rust,ignore
#[derive(ConvertSaveload, Clone, Serialize, Deserialize)]
pub struct Position {
    x: f32,
    y: f32,
}
```

---

## まとめ

- **Specs の `Saveload` モジュールは `serde` を利用してデータの保存・読み込みを行う**
- **エンティティのシリアライズには `Marker` と `MarkerAllocator` を使用**
- **シリアライズ対象の `Component` は `ConvertSaveload` を実装**
- **データの入出力には `SerializeComponents` / `DeserializeComponents` を使用**
- **`World::system_data()` で `SystemData` を取得してシリアライズ処理を実行可能**

---

次の章では、**Specs のカスタマイズとパフォーマンス最適化** について学びます。