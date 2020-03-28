SERVER_CRATE=../server

build: src
	cargo watch \
		-w src \
		-w styles \
		-s "\
			wasm-pack build --target web && \
	    rollup ./main.js --format iife --file ./pkg/bundle.js\
				" &

test: build
	cargo test

serve: build
	cargo watch \
		-w $(SERVER_CRATE)/src \
		-x 'run --manifest-path=$(SERVER_CRATE)/Cargo.toml' 
	
clean:
	rm -rf pkg/
	cargo clean

.PHONY: build serve test clean run
