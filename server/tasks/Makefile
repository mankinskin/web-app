
build:
	cargo watch \
		-w ./src \
		-w ./styles \
		-w ../common/ \
		-s "\
			wasm-pack build --target web &&\
	  	rollup ./index.js --format iife --file ./pkg/bundle.js\
		" \

clean:
	rm -rf pkg/
	cargo clean
	
.PHONY: build clean
