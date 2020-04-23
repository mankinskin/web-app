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
	for app in $(APPS); do\
		make -C $$app clean ;\
	done
	cargo clean
	rm target -r

.PHONY: build serve test clean run
