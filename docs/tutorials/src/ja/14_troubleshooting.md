以下は **"Troubleshooting"（トラブルシューティング）** のチュートリアルの日本語訳です。  
対象読者はプログラマであり、**Specs** はクレートの名称であることを考慮しています。

---

# トラブルシューティング（Troubleshooting）

## **エラー: `Tried to fetch a resource, but the resource does not exist.`**

これは **Specs を初めて使う際に最もよく発生するエラー** です。  
このパニック（エラー）は、`System` が **最初に `dispatch` されるとき** に発生し、  
**必要なコンポーネントまたはリソースが `World` に存在しない** 場合に起こります。

---

## **考えられる原因と解決策**

### **1. `setup()` の呼び出しを忘れている**
- **`Dispatcher` または `ParSeq` を作成した後に `setup()` を実行していない**
- `setup()` は **`System` に必要なコンポーネントやリソースを自動登録する** ため、  
  **最初の `dispatch()` の前に必ず実行する**

✅ **解決策**
```rust,ignore
let mut dispatcher = DispatcherBuilder::new()
    .with(MySystem, "my_system", &[])
    .build();

dispatcher.setup(&mut world); // `setup()` を忘れずに実行
dispatcher.dispatch(&mut world);
```

---

### **2. `World` に必須リソースを追加していない**
- **`ReadExpect<T>` または `WriteExpect<T>` でリソースを要求しているのに、追加していない**
- `ReadExpect<T>` / `WriteExpect<T>` は **リソースが存在しない場合にパニックする**

✅ **解決策**
- `Read<T>` / `Write<T>` を使用する（デフォルト値をセットできる）
- `setup()` 内で `World` にリソースを追加する

```rust,ignore
// `World` にリソースを追加する
world.insert(MyResource::default());
```

---

### **3. `System` 外で `World` からリソースを手動取得している**
- **`System` 以外の場所（例えば `EntityBuilder` の中など）で、リソースを手動で取得**
- **そのリソースが `System` 内で使用されていない**
  - `setup()` は **`System` 内で使用されているコンポーネントやリソースのみを登録** するため、  
    **明示的に `World` に追加する必要がある**

✅ **解決策**
- **`World` に明示的にリソースを登録**
- **全ての `Component` を `World` に登録**
- **`setup()` を適切に実行**

```rust,ignore
// `World` にコンポーネントを登録
world.register::<MyComponent>();

// `World` にリソースを追加
world.insert(MyResource::default());
```

---

## **まとめ**
エラー **"Tried to fetch a resource, but the resource does not exist."** は、
主に **以下の3つの原因** で発生します。

| 原因 | 解決策 |
|------|--------|
| `setup()` の呼び出し忘れ | `setup()` を `dispatch()` 前に実行 |
| `World` にリソースを追加していない | `world.insert(MyResource::default());` を追加 |
| `System` 外で `World` からリソースを取得している | `world.register::<Component>()` を明示的に実行 |

✅ **最初の `dispatch()` の前に `setup()` を必ず実行！**  
✅ **`ReadExpect<T>` や `WriteExpect<T>` を使うなら、`World` にリソースを必ず追加！**  
✅ **`System` 以外で `World` を手動取得する場合、必要なコンポーネントを `register()`！**  

---

これで **Specs でのリソースエラーの原因と対策** を理解できました。  
次の章では、**さらなるデバッグ方法とパフォーマンス最適化** について解説します。