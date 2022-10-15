# Splatoon3TableturfBattle

スプラトゥーン3のナワバトラーの検討用ツール

## 使い方

1. `card_catalog.json` をリポジトリ直下に配置する
2. `bot/src/main.rs` を編集する
3. `cargo run --release --bin judge` でbot同士の対戦が行われる

## カードカタログ(card_catalog.json)の書式

JSON形式で下記の形式。

```json
[
    {
        "id": 1,
        "name": "カード名",
        "cost": 5,
        "squares": "........\n........\n.yyy....\n.yy.Yy..\n..y.....\n.y......\n........\n........"
    },
    ...
]
```

|項目|説明|
|--|--|
|id|カードのID。|
|name|カードの名前。|
|cost|スペシャルアタックを使う際に必要なスペシャルポイント数。|
|squares|カードの形を表現する7x7の文字列。`Y`がスペシャルマス、`y`が通常のマス、`.`が空きマス。|

## TODO
- 仕様確認：スペシャルアタックで塗ったマスに同ターンでマス数の少ないカードで上塗りできるのか？

