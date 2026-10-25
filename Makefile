# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

.DEFAULT_GOAL := help

ADDLICENSE_VERSION := v1.2.0
# Installed here, at this version, so that every machine checks with the same one.
ADDLICENSE := $(CURDIR)/.bin/addlicense-$(ADDLICENSE_VERSION)/addlicense
# The release's own binary for this machine, named as the release names it.
ADDLICENSE_PLATFORM := $(subst Darwin,macOS,$(shell uname -s))_$(subst aarch64,arm64,$(shell uname -m))
ADDLICENSE_ARCHIVE := addlicense_$(ADDLICENSE_VERSION)_$(ADDLICENSE_PLATFORM).tar.gz
# From the release's checksums.txt, pinned here so that a replaced asset is refused.
ADDLICENSE_SHA256_Linux_x86_64 := 6924296771234b1ba6c969e109b4459cff75e15b259676b61df874a92734b242
ADDLICENSE_SHA256_Linux_arm64 := b14dde867a6dbcd41fdf45096831bc0f6500bf0134693d38c56701e179f3f353
ADDLICENSE_SHA256_macOS_x86_64 := 335b47a28bab66e39494222f711c22b140d14d6f6d0a08ab9ea7c6f315588c83
ADDLICENSE_SHA256_macOS_arm64 := 0597305c619f7349748e830e38322dbe7d13cf86cf415b289c70f5cecbb4bf63
ADDLICENSE_SHA256 := $(ADDLICENSE_SHA256_$(ADDLICENSE_PLATFORM))
SHA256SUM := $(if $(shell command -v sha256sum),sha256sum,shasum -a 256)

.PHONY: help dev build test schema-check lint fmt fmt-check license-add license-check check \
	candidates candidates-test

help: ## List available targets
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'

dev: build ## Start the development environment (builds the crate)

build: ## Compile the crate
	cargo build

test: ## Run the test suite
	cargo test

# The data file's cross-references (each source a row names is listed, and so
# on) are checked by build.rs, so every target that builds the crate checks them.
schema-check: ## Check that the data file fits its schema
	cargo test --test schema

lint: ## Lint with clippy, treating warnings as errors
	cargo clippy --all-targets -- -D warnings

fmt: ## Format the code
	cargo fmt

fmt-check: ## Check formatting without modifying files
	cargo fmt --check

$(ADDLICENSE):
	@test -n "$(ADDLICENSE_SHA256)" \
		|| { echo "No addlicense binary pinned for $(ADDLICENSE_PLATFORM)" >&2; exit 1; }
	mkdir -p $(dir $(ADDLICENSE))
	curl -fsSL -o $(dir $(ADDLICENSE))$(ADDLICENSE_ARCHIVE) \
		https://github.com/google/addlicense/releases/download/$(ADDLICENSE_VERSION)/$(ADDLICENSE_ARCHIVE)
	cd $(dir $(ADDLICENSE)) && echo "$(ADDLICENSE_SHA256)  $(ADDLICENSE_ARCHIVE)" | $(SHA256SUM) -c -
	tar -xzf $(dir $(ADDLICENSE))$(ADDLICENSE_ARCHIVE) -C $(dir $(ADDLICENSE)) addlicense
	rm $(dir $(ADDLICENSE))$(ADDLICENSE_ARCHIVE)

license-add: $(ADDLICENSE) ## Add the MPL notice to each tracked source file that has none
	git ls-files -z | xargs -0 $(ADDLICENSE) -l mpl

# addlicense checks the tracked files of the types it knows (Rust, TOML, YAML)
# for a licence header, and passes over the rest (Cargo.lock, Markdown, LICENSE).
# It does not recognise a Makefile, so it does not check this one.
license-check: $(ADDLICENSE) ## Check that each source file carries a licence header
	git ls-files -z | xargs -0 $(ADDLICENSE) -check -l mpl

# test runs the schema check too (tests/schema.rs), so check does not name it.
check: lint fmt-check license-check test ## Run all checks: lint, format check, licence headers, tests (with the schema check)

# --- Temporary: compiling batches of rows (#8) -------------------------------
#
# Removed with tools/compile-batches/ once the table's input is compiled (see
# its README.md). Nothing above depends on these targets; `check` does not run
# them.

N ?= 50

candidates: ## (Temporary) Print the next N designators to compile (N=50; REGISTER=file, REFRESH=1)
	python3 -I tools/compile-batches/candidates.py -n $(N)$(if $(REGISTER), --register "$(REGISTER)")$(if $(REFRESH), --refresh)

candidates-test: ## (Temporary) Test the batch tooling's parser of JO 7360.1's Appendix A
	python3 -I tools/compile-batches/test_appendix_a.py
