# aircraft-type-descriptions

A table of aircraft types, by type designator (`C172`, `PA44`), giving what is
true of each type in every country: the kind of airframe, the number and kind
of its engines, and its wake turbulence category, with the sources each row's
facts come from. It is a data file, which can be read and changed without
reading Rust, and a Rust crate that turns the file into the table when it is
built.

It is for aviation software, such as logbooks and training records, that needs
to know what a type designator describes. Reading those facts under one
country's rules is not done here: CASA's (Australia's) is
[au-casa](https://github.com/Open-Aviation-Solutions/au-casa), which builds on
this crate.

## Status

Early: the version is `0.1.0`. Until `1.0`, the API may change between any two
versions, patch releases included, so pin the version exactly.

What is covered:

- **The table** (`aircraft-type-descriptions.toml`): a small table of
  designators that grows as types are needed, moved from au-casa. Each row
  gives the models the designator covers, its kind of airframe, the number and
  kind of its engines, the sources those facts come from, and notes on what is
  left to check. A row is `confirmed` when someone has read a type certificate
  data sheet for it, and `provisional` otherwise.
- **Designators not to be used**, such as `PA28`, each with what replaces it.
- **The crate** (`DesignatorTable::packaged()`, `lookup`): the table, built
  from the file when the crate is built, so no TOML is parsed when it runs.
- **The schema** (`aircraft-type-descriptions.schema.json`), which CI checks
  the file against, so a mistyped value is caught here and not by a consumer.

Known not to be done yet:

- No row has its wake turbulence category yet: the field is there, and is
  filled in as each row's maximum take-off mass is read from its sources.
- Most designators are not in the table, `C700` among them
  ([au-casa#7](https://github.com/Open-Aviation-Solutions/au-casa/issues/7)).

## Where the facts come from

The type designators are used as keys, as an airport's code is. Each row's
facts come from the sources it names, which are listed once in the file. No
list, table or description code of ICAO Doc 8643 is copied here, and Doc 8643
is not used as a source, or to check a row. The rows moved from au-casa were
compiled before that rule: some of their designator-to-model identifications
had been checked against Doc 8643's published entries there (its tasks `0003`
and `0004`). Their facts rest on the sources they cite.

It is not a type certificate, and nothing in it replaces one. A
`provisional` row has not been checked against a type certificate data sheet.

## Usage

```rust
use aircraft_type_descriptions::{AirframeKind, Confidence, DesignatorTable};

let table = DesignatorTable::packaged();
let c172 = table.lookup("C172").expect("in the table");
assert_eq!(c172.description.airframe, AirframeKind::LandPlane);
assert_eq!(c172.confidence, Confidence::Confirmed);

// Not a current designator: the table says what replaces it.
assert!(table.lookup("PA28").is_none());
assert!(table.not_to_be_used("PA28").unwrap().replaced_by.contains(&"P28A"));
```

## Adding a designator

Add a row under `[designators]` in `aircraft-type-descriptions.toml`, and its
sources under `[sources]` if they are not there yet, each in its place in
alphabetical order. Name a source in `checked-against` only once you have
read it for the row. `make check` checks the file against the schema, and the
build checks that the sources a row names are listed and that the order is
kept.

## Reporting a problem, and contributing

Report a problem, or propose a change, in this repository's
[issues](https://github.com/Open-Aviation-Solutions/aircraft-type-descriptions/issues).

## Development

Requires a Rust toolchain, a C linker (`build-essential` on Debian/Ubuntu) and
`curl`. The first `make check` downloads
[addlicense](https://github.com/google/addlicense)'s release binary into
`.bin/`, and checks it against the checksum pinned in the `Makefile`.

```sh
make help       # list targets
make dev        # build
make test       # run the test suite
make check      # clippy, format check, licence headers, schema, tests
```

See `INSTRUCTIONS.md` for working conventions.

## Licence

[Mozilla Public License 2.0](LICENSE). Each source file carries the licence's
notice.
