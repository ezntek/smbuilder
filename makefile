buildrelease: build_uis
	cargo build --release

runrelease: build_release
	target/release/smbuilder

build: build_uis
	cargo build

run: build
	target/debug/smbuilder

builduis:
	scripts/build_uis.py ui_blueprints ui_xml

installdeps:
	$(shell /usr/bin/env python3 -m pip install requests)
	scripts/install_deps.py