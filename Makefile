
target:
	cargo build

test: target
	cargo test

serve: target
	simple-http-server
