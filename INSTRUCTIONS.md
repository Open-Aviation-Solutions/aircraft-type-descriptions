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

## The data file and the crate

- `aircraft-type-descriptions.toml` is the table, written by hand: sources
  listed once under `[sources]`, rows under `[designators]` keyed by type
  designator, and strings known not to be designators under
  `[not-to-be-used]`. Prose about a row goes in its `notes`; a note about one
  of its sources goes in `source-notes`, keyed by the source's id; a note true
  of a source for every row goes on the source.
- `aircraft-type-descriptions.schema.json` (JSON Schema, draft 2020-12) says
  what each field holds; `make schema-check`, part of `make check`, checks the
  file against it. What a schema cannot say (each source a row names is
  listed; `checked-against` and `source-notes` name only the row's own
  sources; `schema-version` is the one the crate reads) is checked by
  `build.rs`, so a file that breaks it does not build.
- `build.rs` turns the file into Rust statics, which `src/table.rs` includes.
  `toml` is a build-dependency only: nothing parses the file when the crate
  runs, and no TOML parser is in the crate, or in a WebAssembly build of
  anything that depends on it. Keep it so.
- **Confidence is not written; it follows from the sources.** A row is
  `confirmed` when `checked-against` names a source of kind
  `type-certificate-data-sheet`, and `provisional` otherwise. Name a source in
  `checked-against` only when someone has read it for that row.
- **ICAO Doc 8643 is not used or cited**: not as a source, and not to check
  or flag a row (au-casa#7, au-casa#10). No list, table or description code of
  it is copied here. `GLID` cites SKYbrary's article on type designators, by
  as decided on au-casa#7; that is an exception for that row.
- The table is a value (`DesignatorTable::packaged()`), not a global, so that
  a table from elsewhere can be added later without changing what a caller of
  `lookup` sees. Keep `lookup` returning `None` for a designator it does not
  have, rather than guessing.
- **Layer A is narrowed in two places, both documented** in
  `src/description.rs`: `AirframeKind` carries `Glider` and `Airship`, which
  are not among an aircraft description's letters, and
  `EngineType::is_gas_turbine` is there for a Part 61 design feature. Do not
  narrow the types to suit one country without saying so there, since a
  consumer cannot see what is missing.

## Conventions

- **Discuss before implementing**: a design decision is discussed in an issue
  first, and the issue records what was decided.
- **The MPL notice heads each file** in `src/`, `tests/`, `build.rs`, the
  data file, the `Makefile`, `Cargo.toml` and `.github/workflows/` (the
  licence's Exhibit A, in that file's comment syntax; JSON has none, so the
  schema says it in its `description`); `make license-add` adds it to each tracked file
  that has none (so `git add` a new file first). `make license-check`, part
  of `make check`, runs `addlicense -check` over the tracked files, which
  fails on a Rust, TOML or YAML file with no licence header, though it does
  not check that the header is the MPL's wording. Neither covers the
  `Makefile`, which addlicense does not recognise.

## Commands

```sh
make dev            # build the crate
make test           # cargo test
make schema-check   # check the data file against its schema
make lint           # cargo clippy -- -D warnings
make fmt            # cargo fmt
make license-add    # add the MPL notice where a tracked file has none
make license-check  # addlicense -check over the tracked files
make check          # lint + fmt check + licence headers + schema + test
```
