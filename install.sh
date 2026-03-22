mkdir -p out/
cp static-web/index.html static-web/style.css out/
cp -r assets out/
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web --out-dir ./out/ --out-name mygame ./target/wasm32-unknown-unknown/release/momoe.wasm
cd ./out/; simple-http-server -p 8888 --cors
