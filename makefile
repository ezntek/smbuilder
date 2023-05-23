buildrelease:
	cargo build --release

runrelease: build_release
	target/release/smbuilder

build:
	cargo build

run: build
	target/debug/smbuilder