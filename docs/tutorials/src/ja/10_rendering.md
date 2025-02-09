以下は **"Rendering"（レンダリング）** のチュートリアルの日本語訳です。  
対象読者はプログラマであり、**Specs** はクレートの名称であることを考慮しています。

---

# レンダリング（Rendering）

**マルチスレッドの ECS でレンダリングを行うのは少し難しい** ことが多いです。  
そこで、Specs には **「スレッドローカルシステム（thread-local systems）」** という仕組みが用意されています。

---

## スレッドローカルシステムとは？

スレッドローカルシステムには **2 つの重要なルール** があります。

1. **常に `dispatch` の最後に実行される**
2. **依存関係を持てない**（実行順序は `DispatcherBuilder` に追加した順番）

スレッドローカルシステムを追加するには、  
**`DispatcherBuilder::with_thread_local()` を使用** します。

```rust,ignore
DispatcherBuilder::new()
    .with_thread_local(RenderSys);
```

> **この仕組みを利用することで、レンダリングを ECS の一部として統合しやすくなります。**

---

## Amethyst でのレンダリング

[Amethyst](https://amethyst.rs/) では、**Specs がすでに統合されている** ため、  
**特別な処理をする必要はありません**。  
公式のサンプルを見れば、どのように ECS を使ってレンダリングしているかが分かります。

---

## Piston でのレンダリング

[Piston](https://github.com/PistonDevelopers/piston) には、  
次のような **イベントループ** があります。

```rust,ignore
while let Some(event) = window.poll_event() {
    // イベント処理
}
```

ECS を活用するために、**入力情報をリソースとして格納** するのが理想的です。  
例えば、ウィンドウサイズの変更イベントを `ResizeEvents` というリソースに保存できます。

```rust,ignore
struct ResizeEvents(Vec<(u32, u32)>);

world.insert(ResizeEvents(Vec::new()));

while let Some(event) = window.poll_event() {
    match event {
        Input::Resize(x, y) => world.write_resource::<ResizeEvents>().0.push((x, y)),
        // ...
    }
}
```

また、**`Input::Update` イベントが発生したタイミングで `dispatch()` を実行** すると、  
**入力処理と ECS をうまく統合することができます**。

---

## まとめ

- **スレッドローカルシステムを使うと、ECS におけるレンダリングをシンプルに実装できる**
- **スレッドローカルシステムは `dispatch()` の最後に実行され、依存関係を持たない**
- **Amethyst では特別な処理なしに Specs を利用できる**
- **Piston では入力をリソースとして扱い、`dispatch()` を適切に実行するのが推奨される**

---

> **他のゲームエンジンのセクションを追加したい場合は、PR を送ってください！**