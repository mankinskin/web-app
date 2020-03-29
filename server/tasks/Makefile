
build:
		cargo watch \
			-w ./src \
			-w ./styles \
			-s "\
				wasm-pack build --target web && \
		    rollup ./main.js --format iife --file ./pkg/bundle.js\
					" &\

clean:
	rm pkg/ -r
	
.PHONY: build clean
