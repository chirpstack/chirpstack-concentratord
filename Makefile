VERSION ?= $(shell git describe --always |sed -e "s/^v//")

build: version build-armv5-release build-armv7hf-release

package: build package-kerlink package-multitech

version:
	sed -i 's/^version.*/version = "$(VERSION)"/g' ./chirpstack-concentratord-2g4/Cargo.toml
	sed -i 's/^version.*/version = "$(VERSION)"/g' ./chirpstack-concentratord-sx1301/Cargo.toml
	sed -i 's/^version.*/version = "$(VERSION)"/g' ./chirpstack-concentratord-sx1302/Cargo.toml
	sed -i 's/^version.*/version = "$(VERSION)"/g' ./gateway-id/Cargo.toml

clean:
	rm -rf dist

build-native-debug:
	docker-compose run --rm chirpstack-concentratord cargo build

build-native-release:
	docker-compose run --rm chirpstack-concentratord cargo build --release

build-armv5-debug:
	docker-compose run -e BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/arm-linux-gnueabi" --rm chirpstack-concentratord cargo build --target armv5te-unknown-linux-gnueabi

build-armv5-release:
	docker-compose run -e BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/arm-linux-gnueabi" --rm chirpstack-concentratord cargo build --target armv5te-unknown-linux-gnueabi --release

build-armv7hf-debug:
	docker-compose run -e BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/arm-linux-gnueabihf" --rm chirpstack-concentratord cargo build --target arm-unknown-linux-gnueabihf

build-armv7hf-release:
	docker-compose run -e BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/arm-linux-gnueabihf" --rm chirpstack-concentratord cargo build --target arm-unknown-linux-gnueabihf --release

build-aarch64-debug:
	docker-compose run -e BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/aarch64-linux-gnu" --rm chirpstack-concentratord cargo build --target aarch64-unknown-linux-gnu

build-aarch64-release:
	docker-compose run -e BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/aarch64-linux-gnu" --rm chirpstack-concentratord cargo build --target aarch64-unknown-linux-gnu --release

package-multitech: package-multitech-conduit package-multitech-conduit-ap

package-kerlink: package-kerlink-ifemtocell

package-multitech-conduit:
	mkdir -p dist/multitech/conduit
	rm -f packaging/vendor/multitech/conduit/*.ipk
	docker-compose run --rm chirpstack-concentratord bash -c 'cd packaging/vendor/multitech/conduit && ./package.sh ${VERSION}'
	cp packaging/vendor/multitech/conduit/*.ipk dist/multitech/conduit

package-multitech-conduit-ap:
	mkdir -p dist/multitech/conduit-ap
	rm -f packaging/vendor/multitech/conduit-ap/*.ipk
	docker-compose run --rm chirpstack-concentratord bash -c 'cd packaging/vendor/multitech/conduit-ap && ./package.sh ${VERSION}'
	cp packaging/vendor/multitech/conduit-ap/*.ipk dist/multitech/conduit-ap

package-kerlink-ifemtocell:
	mkdir -p dist/kerlink/ifemtocell
	docker-compose run --rm chirpstack-concentratord bash -c 'cd packaging/vendor/kerlink/ifemtocell && ./package.sh ${VERSION}'
	cp packaging/vendor/kerlink/ifemtocell/*.ipk dist/kerlink/ifemtocell

test:
	docker-compose run --rm chirpstack-concentratord cargo test

devshell:
	docker-compose run --rm chirpstack-concentratord bash
