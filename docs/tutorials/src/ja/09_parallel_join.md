# 並列 `join()`（Parallel Join）

[第 3 章][c3] の **Dispatcher の説明** で触れたように、  
Specs は **システムの実行を自動的に並列化** します。  
ただし、**`SystemData` に競合がない場合に限ります**。

- **競合する `SystemData` とは？**
  - **`Read`（読み取り専用）同士 → 並列実行可能**
  - **`Write`（書き込み可能）が 1 つでもある → 競合が発生し、並列実行できない**

[c3]: ./03_dispatcher.html

---

## 基本的な並列化

**Specs は `Dispatcher` 内の並列実行を自動化します** が、  
**システム内の `join()` はデフォルトでは並列実行されません**。

```rust,ignore
fn run(&mut self, (vel, mut pos): Self::SystemData) {
    use specs::Join;
    // このループは単一スレッドで逐次実行される
    for (vel, pos) in (&vel, &mut pos).join() {
        pos.x += vel.x * 0.05;
        pos.y += vel.y * 0.05;
    }
}
```

この処理は **すべてのエンティティに対して逐次的に実行** されるため、  
**数十万のエンティティがあると CPU の並列処理能力を十分に活用できません**。

---

## `par_join()` による並列処理

この問題を解決するために、**`join()` を `par_join()` に変更** します。

```rust,ignore
fn run(&mut self, (vel, mut pos): Self::SystemData) {
    use rayon::prelude::*;
    use specs::ParJoin;

    // `par_join()` に変更すると、内部でスレッドプールを利用して並列実行される
    (&vel, &mut pos)
        .par_join()
        .for_each(|(vel, pos)| {
            pos.x += vel.x * 0.05;
            pos.y += vel.y * 0.05;
        });
}
```

### **`par_join()` のポイント**
- **`join()` と同じように使える**
- **内部で `rayon` のスレッドプールを利用し、並列処理を実現**
- **`.for_each()` などの `ParallelIterator` メソッドが使用可能**

---

## 並列化の注意点

### **並列化にはオーバーヘッドがある**
並列実行には **スレッドの管理コスト** が発生するため、  
**必ずしも `par_join()` が `join()` より高速とは限りません**。

> **`par_join()` を使用する前に、プロファイリングで効果を確認することを推奨** します。

次のような場合は **`join()` のままの方が高速** です。
- **処理するエンティティの数が少ない**
- **ループ処理のコストが非常に低い**

---

## `par_join()` の拡張

**`par_join()` は `rayon` の [`ParallelIterator`][ra] を実装** しており、  
通常の `Iterator` と同じように便利なメソッドを使うことができます。

例えば、**並列でフィルタリングやマップ処理を行う** ことも可能です。

```rust,ignore
(&vel, &mut pos)
    .par_join()
    .filter(|(vel, _)| vel.x.abs() > 0.1 || vel.y.abs() > 0.1) // 小さい速度のエンティティを除外
    .for_each(|(vel, pos)| {
        pos.x += vel.x * 0.05;
        pos.y += vel.y * 0.05;
    });
```

[ra]: https://docs.rs/rayon/1.0.0/rayon/iter/trait.ParallelIterator.html

---

## まとめ

- **`Dispatcher` は `System` の並列実行を自動化するが、`join()` はデフォルトで逐次実行される**
- **`par_join()` を使うと `join()` を並列実行できる**
- **`par_join()` は `rayon` の `ParallelIterator` を実装している**
- **並列化にはオーバーヘッドがあるため、プロファイリングで効果を確認するのが重要**

---

次の章では、**システムのスケジューリング（System Scheduling）** について学びます。