.PHONY: clean

CLEAN_DIRNAMES := target/debug target/release target/doc
CLEAN_DIRS := $(strip $(foreach dir,$(CLEAN_DIRNAMES),$(wildcard $(dir))))

docs:
	cargo doc --release --no-deps

format:
	cargo fmt

check:
	cargo clippy

build:
ifeq ($(BUILDTYPE),release)
	cargo build --release
else
	cargo build
endif

clean: 
ifneq (,$(CLEAN_DIRS))
	rm -r $(CLEAN_DIRS)
endif