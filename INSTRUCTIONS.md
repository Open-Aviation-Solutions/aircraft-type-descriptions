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
- **Rows and sources are in alphabetical order** (byte, ASCII, order of their
  keys, each `source-notes` table directly under its row), so a new row has
  one place to go and two pull requests adding rows rarely touch the same
  lines; the build checks it.
- **One file, until rows conflict.** While the table is small, one file reads
  best, and a new row and its new source are one edit. When pull requests that
  add rows start to conflict, split it as one file per designator
  (`designators/C172.toml`) plus `sources.toml`, so that each new row is a new
  file. Do not split by airframe, engine type or any national category: those
  are facts in a row, not its identity, so correcting one would move the row
  between files. The layout is internal: `build.rs` reads it, and a consumer
  sees only the crate (or, later, the compiled JSON), so splitting changes
  nothing for them. `schema-version` then needs a home of its own
  (`sources.toml`, or a small manifest).
- `aircraft-type-descriptions.schema.json` (JSON Schema, draft 2020-12) says
  what each field holds; `make schema-check` checks the file against it, and
  `make test`, and so `make check`, runs the same test. What a schema cannot
  say (each source a row names is listed; `checked-against` and
  `source-notes` name only the row's own sources; `schema-version` is the one
  the crate reads; the order of the keys) is checked by `build.rs`, so a file
  that breaks it does not build.
- `build.rs` turns the file into Rust statics, which `src/table.rs` includes.
  `toml` is a build-dependency only: nothing parses the file when the crate
  runs, and no TOML parser is in the crate, or in a WebAssembly build of
  anything that depends on it. Keep it so.
- **Confidence is not written; it follows from the sources.** A row is
  `confirmed` when `checked-against` names a source of kind
  `type-certificate-data-sheet`, and `provisional` otherwise. Name a source in
  `checked-against` only when someone has read it for that row. A row read
  against a sheet that covers only some of the designator's models stays
  `confirmed`, and its notes name the models the sheet does not cover.
- **ICAO Doc 8643 is not used or cited**: not as a source, and not to check
  or flag a row (au-casa#7, au-casa#10). No list, table or description code of
  it is copied here. `GLID` cites SKYbrary's article on type designators, as
  decided on au-casa#7; that is an exception for that row. FAA Order JO
  7360.1 is a source in its own right, although its designators and wake
  turbulence categories are ICAO's: it is a US Government work, published
  under the FAA's own authority with its own analysis (its paragraph 2-2b), so
  its class, engine and category columns may be used (decided on #3, 25
  October 2026).
- The table is a value (`DesignatorTable::packaged()`), not a global, so that
  a table from elsewhere can be added later without changing what a caller of
  `lookup` sees. Keep `lookup` returning `None` for a designator it does not
  have, rather than guessing.
- **Layer A has two things in it for Part 61, both documented** in
  `src/description.rs`: `AirframeKind` carries `Glider` and `Airship` as kinds
  of airframe of their own, and `EngineType::is_gas_turbine` is there for a
  Part 61 design feature. Do not
  narrow the types to suit one country without saying so there, since a
  consumer cannot see what is missing.

## Compiling a batch (temporary)

**Temporary, with `tools/compile-batches/`**: this section goes when the
table's input is compiled and the tooling is removed (its README says how).

A batch of rows (#9) starts from `make candidates N=50`, which prints the next
designators by number of aircraft on CASA's register, with what FAA Order JO
7360.1 and the register give for each, and the edition, date and SHA-256 of
the files it read, for the pull request to cite. CASA's server refuses
scripted downloads, so in practice each batch downloads the register's
`acrftreg.csv` in a browser and gives it, with the date CASA's page gives for
it, as `make candidates REGISTER=path REGISTER_DATE=date`. The tooling is Python,
standard library only, plus `pdftotext`, run with `python3 -I`; it is not in
the crate's `include` list, and nothing in the crate, `build.rs` or `make
check` depends on it. Its parser's test is `make candidates-test`. What the
first batch (#6) learnt:

- **JO 7360.1 never supplies a glider's airframe.** It files a self-launching
  glider as fixed-wing, with its engine, so its class does not tell a glider
  from an aeroplane: take a glider's airframe from a data sheet or the
  register. `make candidates` flags a designator the register files gliders
  under.
- **Its engine type `T` is "Turboprop/Turboshaft"**, which is `turboprop`
  here (the schema's `turboprop` covers a turboshaft).
- **The register's "Diesel Engine" and "Rotary" (a Wankel, such as the Austro
  AE50R) are piston engines**, as the order and the table have them (the
  owner's ruling on #11), so they agree with a `piston` row.
- **Light/Medium means the category is left out**, and the row's notes say
  why: the designator's variants fall in more than one category.
- **Turbine conversions have designators of their own** (`B47T`, `H12T`,
  `C06T`), so where the register lists turbine aircraft under a piston
  designator, the row's notes point at the conversion's designator rather
  than describe them.
- **The register is cited only where every aircraft under the designator
  agrees** with the row's number and kind of engines; where even one differs,
  the row does not cite it, and its notes say so.
- **EASA data sheets are paraphrased, never quoted**: they are marked "All
  rights reserved".

## Conventions

- **Discuss before implementing**: a design decision is discussed in an issue
  first, and the issue records what was decided.
- **The MPL notice heads each file** in `src/`, `tests/`, `build.rs`, the data
  file, the `Makefile`, `Cargo.toml`, `.github/workflows/` and the scripts and
  fixture of the temporary `tools/compile-batches/` (the licence's
  Exhibit A, in that file's comment syntax; JSON has none, so the schema says
  it in its `description`); `make license-add` adds it to each tracked file
  that has none (so `git add` a new file first). `make license-check`, part of
  `make check`, runs `addlicense -check` over the tracked files, which fails
  on a Rust, TOML, YAML, Python or HTML file with no licence header, though it does not
  check that the header is the MPL's wording. Neither covers the `Makefile`,
  which addlicense does not recognise. It sees only tracked files, so a new
  file not yet added passes locally and fails in CI; and it passes a tracked
  file it cannot read (deleted but not staged, say).

## Releasing

Bump `version` in `Cargo.toml` in the pull request that changes the data
file, the schema or the crate, update `Cargo.lock` to match
(`cargo update --workspace`; `make check` runs cargo with `--locked`, so a
stale one fails), and merge it: that is the release. Until `1.0`:

- a **patch** for rows and sources added or corrected;
- a **minor** version for a change to the API, to the data file's schema, or
  to how confidence is derived.

On each push to `main`, the Release workflow (`.github/workflows/release.yml`)
asks crates.io whether `Cargo.toml`'s version is there yet. When it is not,
it runs `make check`, publishes it through trusted publishing (no crates.io
token is kept in the repository), and tags the commit `v<version>`. Don't
push version tags by hand: nothing runs on them. The one exception: if a
version is on crates.io but the workflow failed to push its tag, the
maintainer tags that merge commit `v<version>` by hand, since a re-run does
not tag it: with the version already there, it skips publishing, or fails at
`cargo publish`.

Only `main` can release. The publishing job runs in the `release`
environment, which GitHub allows only on `main`, and crates.io's trusted
publisher accepts only that environment; `main` changes only through a pull
request with CI passing.

The first release, `0.1.0`, was published by hand by the maintainer, from
`main`, since crates.io's trusted publishing can only be set up for a crate
it already has, and its tag, `v0.1.0`, was pushed by hand. Every tag after it
comes from the workflow.

What is published is `Cargo.toml`'s `include` list: the crate's source and
tests, `build.rs`, the data file and its schema, the README and the licence.
A file the build or the tests read goes on that list, or the published crate
does not build; `cargo package --list` shows what it holds, and
`cargo publish --dry-run --locked` builds it as crates.io would.

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

# Temporary, with tools/compile-batches/ (see "Compiling a batch")
make candidates N=50  # the next 50 designators to compile (REGISTER=file, REFRESH=1)
make candidates-test  # the test of its parser of JO 7360.1's Appendix A
```
