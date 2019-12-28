build-native-debug:
	cargo build

build-native-release:
	cargo build --release

build-armv7-debug:
	BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/arm-linux-gnueabihf" cargo build --target armv7-unknown-linux-gnueabihf

build-armv7-release:
	BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/arm-linux-gnueabihf" cargo build --target armv7-unknown-linux-gnueabihf --release
