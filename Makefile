build-native-debug:
	cargo build

build-native-release:
	cargo build --release

build-armv5-debug:
	BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/arm-linux-gnueabi" cargo build --target armv5te-unknown-linux-gnueabi

build-armv5-release:
	BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/arm-linux-gnueabi" cargo build --target armv5te-unknown-linux-gnueabi --release

build-armv7hf-debug:
	BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/arm-linux-gnueabihf" cargo build --target arm-unknown-linux-gnueabihf

build-armv7hf-release:
	BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/arm-linux-gnueabihf" cargo build --target arm-unknown-linux-gnueabihf --release
test:
	cargo test
