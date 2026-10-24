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

Being set up. The table, its schema and the crate's types move here from
au-casa ([au-casa#7](https://github.com/Open-Aviation-Solutions/au-casa/issues/7));
until they do, the crate is empty.

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
make check      # clippy, format check, licence headers, tests
```

See `INSTRUCTIONS.md` for working conventions.

## Licence

[Mozilla Public License 2.0](LICENSE). Each source file carries the licence's
notice.
