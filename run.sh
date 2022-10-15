cargo build --release && RUST_LOG=info cargo run --release --bin judge > tmp/kifu.txt

rm tmp/TableturfBattle_*.svg tmp/TableturfBattle_*.png

RUST_LOG=info cargo run --release --bin record_player < tmp/kifu.txt
convert -density 50 tmp/TableturfBattle_*.svg tmp/TableturfBattle_%04d.png
convert -delay 50 -loop 0 tmp/TableturfBattle_*.png tmp/TableturfBattle.gif
