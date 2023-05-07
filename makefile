build_release: build_uis
	cargo build --release

run_release: build_release
	target/release/smbuilder

build: build_uis
	cargo build

run: build
	target/debug/smbuilder

build_uis:
	scripts/build_uis.py ui_blueprints ui_xml