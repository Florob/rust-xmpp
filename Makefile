RUSTC?=rustc

RUSTCFLAGS=-O -L src/rust-openssl/build/ -L src/RustyXML/build/ -L build/ --out-dir=build/

alL: example

lib: build
	$(RUSTC) $(RUSTCFLAGS) src/xmpp/lib.rs

example: lib
	$(RUSTC) $(RUSTCFLAGS) src/example/main.rs

build:
	mkdir build

clean:
	rm -rf build/

.PHONY: all clean lib example
