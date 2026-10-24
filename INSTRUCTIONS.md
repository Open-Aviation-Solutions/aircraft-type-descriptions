# aircraft-type-descriptions — Working Instructions

The country-agnostic description of aircraft types, by type designator (the
"Layer A" that au-casa used to hold): a data file of rows, each citing its
sources, and a Rust crate holding the types and building the table from the
file. It moved out of au-casa because none of it is Australian; a New Zealand
or United Kingdom package would need it unchanged
([au-casa#7](https://github.com/Open-Aviation-Solutions/au-casa/issues/7)).

## Purpose and scope

In scope: what is true of an aircraft type wherever it flies (the kind of
airframe, the number and kind of engines, the wake turbulence category), the
sources each fact comes from, and the designators known not to be in use.

Out of scope: reading those facts under one country's rules (a licensing
category, class or design feature). That belongs to a national package, such
as au-casa for CASA's Part 61.

## Conventions

- **Discuss before implementing**: a design decision is discussed in an issue
  first, and the issue records what was decided.
- **The MPL notice heads each file** in `src/`, `tests/`, the `Makefile`,
  `Cargo.toml` and `.github/workflows/` (the licence's Exhibit A, in that
  file's comment syntax); `make license-add` adds it to each tracked file
  that has none (so `git add` a new file first). `make license-check`, part
  of `make check`, runs `addlicense -check` over the tracked files, which
  fails on a Rust, TOML or YAML file with no licence header, though it does
  not check that the header is the MPL's wording. Neither covers the
  `Makefile`, which addlicense does not recognise. It sees only tracked
  files, so a new file not yet added passes locally and fails in CI; and it
  passes a tracked file it cannot read (deleted but not staged, say).

## Commands

```sh
make dev            # build the crate
make test           # cargo test
make lint           # cargo clippy -- -D warnings
make fmt            # cargo fmt
make license-add    # add the MPL notice where a tracked file has none
make license-check  # addlicense -check over the tracked files
make check          # lint + fmt check + licence headers + test
```
