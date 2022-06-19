# This is free and unencumbered software released into the public domain.

# Makefile for building oci-run
#
# Optional variables:
#
# - CARGO: path to the cargo binary
# - CARGO_CLIPPY_OPTS: options passed to clippy
# - CARGO_CLIPPY_OPTS_EXTRA: additional options passed to clippy
# - CARGO_DOC_OPTS: options passed to cargo doc
# - CARGO_DOC_OPTS_EXTRA: additional options passed to cargo doc
# - CARGO_OPTS: options passed to cargo

CARGO ?= cargo
CARGO_OPTS ?=
CARGO_CLIPPY_OPTS ?= -D warnings -D clippy::all -D clippy::pedantic $(CARGO_CLIPPY_OPTS_EXTRA)
CARGO_DOC_OPTS ?= --no-deps $(CARGO_DOC_OPTS_EXTRA)

# Target recipes
.DEFAULT_GOAL := all
.PHONY: all build clean compile doc release test

all: test build doc

build: test compile

clean:
	$(CARGO) clean

compile:
	$(CARGO) build $(CARGO_OPTS)

doc:
	$(CARGO) doc $(CARGO_OPTS) $(CARGO_DOC_OPTS)

release: CARGO_OPTS += --release
release: all

test:
	$(CARGO) fmt -- --check
	$(CARGO) clippy $(CARGO_OPTS) -- $(CARGO_CLIPPY_OPTS)
	$(CARGO) test $(CARGO_OPTS)
