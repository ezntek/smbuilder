build_release:
	cargo build --release

run_release: build_release
	target/release/smbuilder

build:
	cargo build

run: build
	target/debug/smbuilder

build_uis:
	scripts/build_uis.py ui_blueprints ui_xml