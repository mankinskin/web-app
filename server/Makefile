
client:
		make -C client

run: client
	cargo run 
	
clean:
	make -C client clean
	cargo clean
	rm target -r

.PHONY: build serve clean client
