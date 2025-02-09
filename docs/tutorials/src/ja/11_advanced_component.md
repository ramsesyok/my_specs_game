# コンポーネントの高度な戦略（Advanced Strategies for Components）

Specs の基本を理解したので、  
**より高度なパターン** を試してみましょう！

---

## マーカーコンポーネント（Marker Components）

特定のエンティティに対して **追加の処理を行いたい場合**、  
**マーカーコンポーネント** を使うのが一般的な方法です。

例えば、**一部のエンティティにのみ空気抵抗（Drag Force）を適用したい** ケースを考えます。  
`Velocity` を持つすべてのエンティティに空気抵抗を適用するのではなく、  
特定のエンティティのみに適用するようにします。

### **マーカーコンポーネントの例**
```rust,ignore
#[derive(Component)]
#[storage(NullStorage)]
pub struct Drag; // ← マーカーコンポーネント（データなし）

#[derive(Component)]
pub struct Position {
    pub pos: [f32; 3],
}

#[derive(Component)]
pub struct Velocity {
    pub velocity: [f32; 3],
}

struct Sys {
    drag: f32,
}

impl<'a> System<'a> for Sys {
    type SystemData = (
        ReadStorage<'a, Drag>,
        ReadStorage<'a, Velocity>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, (drag, velocity, mut position): Self::SystemData) {
        // 空気抵抗があるエンティティ
        for (pos, vel, _) in (&mut position, &velocity, &drag).join() {
            pos += vel - self.drag * vel * vel;
        }
        // 空気抵抗がないエンティティ
        for (pos, vel, _) in (&mut position, &velocity, !&drag).join() {
            pos += vel;
        }
    }
}
```

### **ポイント**
- **`Drag` コンポーネントはデータを持たず、エンティティを「マーク」するだけ**
- **`NullStorage` を使用すると、メモリ消費ゼロで管理可能**
- **`!&drag` を使うと `Drag` を持たないエンティティを対象にできる**

> **⚠️ `NullStorage` は ZST（Zero-Sized Type：フィールドを持たない構造体）でのみ動作します！**

---

## エンティティの関係・階層構造のモデル化

エンティティ間の関係を表現するケースは多くあります。  
例えば、**三人称視点のカメラがプレイヤーを追尾する場合**、  
カメラはプレイヤーエンティティをターゲットとして参照する必要があります。

### **エンティティのターゲティング**
```rust,ignore
#[derive(Component)]
pub struct Target {
    target: Entity,
    offset: Vector3,
}

pub struct FollowTargetSys;

impl<'a> System<'a> for FollowTargetSys {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Target>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (entity, target, transform): Self::SystemData) {
        for (entity, t) in (&*entity, &target).join() {
            let new_transform = transform.get(t.target).cloned().unwrap() + t.offset;
            *transform.get_mut(entity).unwrap() = new_transform;
        }
    }
}
```

### **この設計のメリット**
- **複数のエンティティが異なるターゲットを追従可能**
- **ターゲットの変更が簡単**
- **大規模な階層構造（シーングラフ）のモデル化にも応用可能**

> より一般的な階層管理のために、[`specs-hierarchy`](https://github.com/rustgd/specs-hierarchy) クレートをチェックしてください。

---

## 観戦モードのエンティティターゲティング

チーム制 FPS ゲームを作る場合、**観戦モード** で特定のプレイヤーをフォローする機能が必要になります。  
この場合、観戦カメラがフォローするプレイヤーを切り替えられるようにします。

### **ターゲットエンティティをリソースとして管理**
```rust,ignore
pub struct ActiveCamera(Entity);

pub struct Render;

impl<'a> System<'a> for Render {
    type SystemData = (
        Read<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Mesh>,
    );

    fn run(&mut self, (active_cam, camera, transform, mesh) : Self::SystemData) {
        let camera = camera.get(active_cam.0).unwrap();
        let view_matrix = transform.get(active_cam.0).unwrap().invert();
        // ビュー行列と投影行列を設定
        for (mesh, transform) in (&mesh, &transform).join() {
            // ワールド変換行列を設定
            // メッシュをレンダリング
        }
    }
}
```

### **ポイント**
- **`ActiveCamera(Entity)` をリソースとして管理**
- **観戦対象のプレイヤーを変更すると `ActiveCamera` の参照を更新**
- **描画システムは `ActiveCamera` で指定されたエンティティの視点を使用**

---

## コンポーネントの値でエンティティをソートする

多くの場面で、**特定のコンポーネントの値を基準にエンティティをソートしたい** ことがあります。

### **基本的なソート**
```rust,ignore
let mut data = (&entities, &comps).join().collect::<Vec<_>>();
data.sort_by(|a, b| a.1.value.partial_cmp(&b.1.value).unwrap());

for entity in data.iter().map(|d| d.0) {
    // ソートされた順番でエンティティを処理
}
```

### **この方法の制約**
1. **毎フレームすべてのエンティティを処理する**
   - `System` でソートする場合、**毎フレーム `Vec` を作成＆ソートするコスト** がかかる。
2. **毎フレーム `Vec` を新しく確保する**
   - `Vec` の確保コストを減らすには、`System` 内部に `Vec` をキャッシュして再利用するのが良い。

これらの制約を解決するために、**`FlaggedStorage` を活用してソート済みのエンティティリストを維持** する方法もあります。  
詳細は次の [FlaggedStorage の章][fs] で解説します。

[fs]: ./12_tracked.html

---

## まとめ

- **マーカーコンポーネント（データなし）を使うと、エンティティの分類が簡単**
  - `NullStorage` を使うとメモリ消費ゼロ
- **エンティティ間の関係をモデル化するには、ターゲットコンポーネントを利用**
  - 三人称視点カメラや階層構造の管理に便利
- **観戦モードでは、ターゲットエンティティをリソースとして管理**
  - `ActiveCamera(Entity)` をリソースとして保持すると、カメラの切り替えが簡単
- **エンティティをソートする場合、毎フレーム `Vec` を確保するコストを考慮**
  - `FlaggedStorage` を使えばソート済みリストを維持できる（次章で解説）

---

次の章では、**`FlaggedStorage` を活用したエンティティの追跡と変更検知** について学びます。