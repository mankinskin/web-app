build: src
	cargo watch -w src -s "\
	        wasm-pack build --target web && \
	        rollup ./main.js --format iife --file ./pkg/bundle.js" &

test: build
	cargo test

serve: build
	cargo run --manifest-path=../server/Cargo.toml &
	
clean:
	rm -rf pkg/
	cargo clean

.PHONY: build serve test clean run
