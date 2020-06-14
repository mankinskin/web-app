
client:
		make -C client

run: client
	cargo watch \
		-w src \
		-x run 
	
clean:
	make -C client clean
	cargo clean
	rm target -r

.PHONY: build serve clean client
