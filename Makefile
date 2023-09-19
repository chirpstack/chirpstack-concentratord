.PHONY: dist

# Compile the binaries for all targets.
build:
	cross build --target aarch64-unknown-linux-musl --release
	cross build --target armv5te-unknown-linux-musleabi --release
	cross build --target armv7-unknown-linux-musleabihf --release
	cross build --target x86_64-unknown-linux-musl --release

# Update the version.
version:
	test -n "$(VERSION)"
	sed -i 's/^version.*/version = "$(VERSION)"/g' ./chirpstack-concentratord-2g4/Cargo.toml
	sed -i 's/^version.*/version = "$(VERSION)"/g' ./chirpstack-concentratord-sx1301/Cargo.toml
	sed -i 's/^version.*/version = "$(VERSION)"/g' ./chirpstack-concentratord-sx1302/Cargo.toml
	sed -i 's/^version.*/version = "$(VERSION)"/g' ./gateway-id/Cargo.toml
	sed -i 's/^version.*/version = "$(VERSION)"/g' ./libconcentratord/Cargo.toml
	make test
	git add .
	git commit -v -m "Bump version to $(VERSION)"
	git tag -a v$(VERSION) -m "v$(VERSION)"

# Cleanup dist.
clean:
	cargo clean
	rm -rf dist

# Run tests.
test:
	cross clippy --target x86_64-unknown-linux-musl
	cross test --target x86_64-unknown-linux-musl

# Enter the devshell.
devshell:
	nix-shell

# Build distributable binaries.
dist: build package

package: \
	package-targz-armv7hf \
	package-targz-arm64	\
	package-targz-amd64 \
	package-kerlink-ifemtocell \
	package-multitech-conduit \
	package-multitech-conduit-ap

package-kerlink-ifemtocell:
	cd packaging/vendor/kerlink/ifemtocell && ./package.sh
	mkdir -p dist/vendor/kerlink/ifemtocell
	cp packaging/vendor/kerlink/ifemtocell/*.ipk dist/vendor/kerlink/ifemtocell

package-multitech-conduit:
	cd packaging/vendor/multitech/conduit && ./package-sx1301.sh && ./package-sx1302.sh && ./package-2g4.sh
	mkdir -p dist/vendor/multitech/conduit
	cp packaging/vendor/multitech/conduit/*.ipk dist/vendor/multitech/conduit

package-multitech-conduit-ap:
	cd packaging/vendor/multitech/conduit-ap && ./package.sh
	mkdir -p dist/vendor/multitech/conduit-ap
	cp packaging/vendor/multitech/conduit-ap/*.ipk dist/vendor/multitech/conduit-ap

package-targz-armv7hf:
	$(eval PKG_VERSION := $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'))
	mkdir -p dist
	tar -czvf dist/chirpstack-concentratord-sx1301_$(PKG_VERSION)_armv7hf.tar.gz -C target/armv7-unknown-linux-musleabihf/release chirpstack-concentratord-sx1301
	tar -czvf dist/chirpstack-concentratord-sx1302_$(PKG_VERSION)_armv7hf.tar.gz -C target/armv7-unknown-linux-musleabihf/release chirpstack-concentratord-sx1302
	tar -czvf dist/chirpstack-concentratord-2g4_$(PKG_VERSION)_armv7hf.tar.gz -C target/armv7-unknown-linux-musleabihf/release chirpstack-concentratord-2g4

package-targz-arm64:
	$(eval PKG_VERSION := $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'))
	mkdir -p dist
	tar -czvf dist/chirpstack-concentratord-sx1301_$(PKG_VERSION)_arm64.tar.gz -C target/aarch64-unknown-linux-musl/release chirpstack-concentratord-sx1301
	tar -czvf dist/chirpstack-concentratord-sx1302_$(PKG_VERSION)_arm64.tar.gz -C target/aarch64-unknown-linux-musl/release chirpstack-concentratord-sx1302
	tar -czvf dist/chirpstack-concentratord-2g4_$(PKG_VERSION)_arm64.tar.gz -C target/aarch64-unknown-linux-musl/release chirpstack-concentratord-2g4

package-targz-amd64:
	$(eval PKG_VERSION := $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'))
	mkdir -p dist
	tar -czvf dist/chirpstack-concentratord-sx1301_$(PKG_VERSION)_amd64.tar.gz -C target/x86_64-unknown-linux-musl/release chirpstack-concentratord-sx1301
	tar -czvf dist/chirpstack-concentratord-sx1302_$(PKG_VERSION)_amd64.tar.gz -C target/x86_64-unknown-linux-musl/release chirpstack-concentratord-sx1302
	tar -czvf dist/chirpstack-concentratord-2g4_$(PKG_VERSION)_amd64.tar.gz -C target/x86_64-unknown-linux-musl/release chirpstack-concentratord-2g4
