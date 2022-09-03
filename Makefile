deb:
	cargo deb
	mkdir debian
	cp target/debian/* debian/
clean:
	cargo clean
	rm -rf debian
