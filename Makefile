.PHONY: build install install-local release clean

build:
	cargo build --release

install: build
	sudo install -m 755 target/release/easytodo /usr/local/bin/easytodo
	sudo install -m 755 target/release/mcp /usr/local/bin/easytodo-mcp

install-local: build
	mkdir -p ~/.local/bin
	install -m 755 target/release/easytodo ~/.local/bin/easytodo
	install -m 755 target/release/mcp ~/.local/bin/easytodo-mcp
	@echo "Make sure ~/.local/bin is in your PATH"

release:
	cargo build --release

clean:
	cargo clean
