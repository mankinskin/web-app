APPS=home tasks

build: $(APPS)
	for app in $(APPS); do\
		make -C $$app ;\
	done

serve: build
	cargo watch \
		-w src \
		-x run 
	
clean:
	cargo clean

.PHONY: build serve test clean run
