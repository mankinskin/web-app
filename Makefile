build:
	wasm-pack build --target web
	rollup ./main.js --format iife --file ./pkg/bundle.js

test: build
	cargo test

serve: target
	simple-http-server

clean:
	rm -rf pkg/
	cargo clean
