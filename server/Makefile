build: src
	cargo watch -w src -s "\
		wasm-pack build --target web && \
		rollup ./main.js --format iife --file ./pkg/bundle.js" &

test: build
	cargo test

serve: build
	cargo watch -w styles/ -w index.html -w main.js -s "./run_server.sh"

clean:
	rm -rf pkg/
	cargo clean

run:
	cargo run
