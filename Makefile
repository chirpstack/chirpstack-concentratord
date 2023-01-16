.PHONY: dist

build:
	cargo build

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
	rm -rf dist

# Run tests.
test:
	docker-compose run --rm chirpstack-concentratord cargo clippy --no-deps
	docker-compose run --rm chirpstack-concentratord cargo test

# Enter the devshell.
devshell:
	docker-compose run --rm chirpstack-concentratord bash

# Build distributable binaries.
dist:
	docker-compose run --rm chirpstack-concentratord make \
		docker-package-targz-armv7hf \
		docker-package-targz-arm64 \
		docker-package-kerlink-ifemtocell \
		docker-package-multitech-conduit \
		docker-package-multitech-conduit-ap

###
# All docker-... commands must be executed within the Docker Compose environment.
###

docker-release-armv7hf:
	BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/arm-linux-gnueabihf" \
		cargo build --target arm-unknown-linux-gnueabihf --release

docker-release-armv5:
	BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/arm-linux-gnueabi" \
		cargo build --target armv5te-unknown-linux-gnueabi --release

docker-release-arm64:
	BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/aarch64-linux-gnu" \
		cargo build --target aarch64-unknown-linux-gnu --release

docker-package-kerlink-ifemtocell: docker-release-armv7hf
	cd packaging/vendor/kerlink/ifemtocell && ./package.sh
	mkdir -p dist/vendor/kerlink/ifemtocell
	cp packaging/vendor/kerlink/ifemtocell/*.ipk dist/vendor/kerlink/ifemtocell

docker-package-multitech-conduit: docker-release-armv5
	cd packaging/vendor/multitech/conduit && ./package-sx1301.sh && ./package-sx1302.sh && ./package-2g4.sh
	mkdir -p dist/vendor/multitech/conduit
	cp packaging/vendor/multitech/conduit/*.ipk dist/vendor/multitech/conduit

docker-package-multitech-conduit-ap: docker-release-armv5
	cd packaging/vendor/multitech/conduit-ap && ./package.sh
	mkdir -p dist/vendor/multitech/conduit-ap
	cp packaging/vendor/multitech/conduit-ap/*.ipk dist/vendor/multitech/conduit-ap

docker-package-targz-armv7hf: docker-release-armv7hf
	$(eval PKG_VERSION := $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'))
	mkdir -p dist
	tar -czvf dist/chirpstack-concentratord-sx1301_$(PKG_VERSION)_armv7hf.tar.gz -C target/arm-unknown-linux-gnueabihf/release chirpstack-concentratord-sx1301
	tar -czvf dist/chirpstack-concentratord-sx1302_$(PKG_VERSION)_armv7hf.tar.gz -C target/arm-unknown-linux-gnueabihf/release chirpstack-concentratord-sx1302
	tar -czvf dist/chirpstack-concentratord-2g4_$(PKG_VERSION)_armv7hf.tar.gz -C target/arm-unknown-linux-gnueabihf/release chirpstack-concentratord-2g4

docker-package-targz-arm64: docker-release-arm64
	$(eval PKG_VERSION := $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'))
	mkdir -p dist
	tar -czvf dist/chirpstack-concentratord-sx1301_$(PKG_VERSION)_arm64.tar.gz -C target/aarch64-unknown-linux-gnu/release chirpstack-concentratord-sx1301
	tar -czvf dist/chirpstack-concentratord-sx1302_$(PKG_VERSION)_arm64.tar.gz -C target/aarch64-unknown-linux-gnu/release chirpstack-concentratord-sx1302
	tar -czvf dist/chirpstack-concentratord-2g4_$(PKG_VERSION)_arm64.tar.gz -C target/aarch64-unknown-linux-gnu/release chirpstack-concentratord-2g4
