# compile-batches (temporary)

**This directory is temporary.** It holds the tooling for compiling the table
in batches of rows
([#8](https://github.com/Open-Aviation-Solutions/aircraft-type-descriptions/issues/8),
for the batches of
[#9](https://github.com/Open-Aviation-Solutions/aircraft-type-descriptions/issues/9)),
and is removed once the table's input is compiled. It is not part of the
crate: it is not in `Cargo.toml`'s `include` list, and nothing in the crate,
`build.rs` or `make check` uses it.

## What it does

`make candidates N=50` prints the next N designators to compile, with what
FAA Order JO 7360.1 and CASA's aircraft register say of each:

- **The order**: its class, engines and FAA weight class, wake turbulence
  category and models (Appendix A), and the row's airframe, engines and
  category as the table would have them.
- **The register**: its number of aircraft under the designator, and whether
  every one has the order's number and type of engines, or how many differ.
- **Flags** (`!`) for a batch to look at: a category of Light/Medium, the
  register disagreeing on engines, and gliders on the register under a
  designator the order calls fixed-wing.

The candidates are the register's designators, by number of aircraft, highest
first and ties by designator, that the order gives FAA weight class S or S+,
less its special designators (such as `GLID`) and the designators already in
the table. It ends with how many candidates are left.

The first run downloads the current edition of the order (from the FAA's page
for it) and the register's `acrftreg.csv` into `cache/`, which git ignores, and
later runs use those copies until `make candidates REFRESH=1`. Each file's URL,
date and SHA-256 are kept in `cache/sources.json` and printed first, so a
batch can cite exactly what it used.

CASA's server refuses some scripted downloads. If it refuses this one,
download `acrftreg.csv` in a browser from CASA's
[data files page](https://www.casa.gov.au/aircraft/aircraft-registration/data-files-registered-aircraft)
and give it with `make candidates REGISTER=path/to/acrftreg.csv`; it is
copied into `cache/` and used from then on.

## What it needs

Python 3.11 or later, standard library only, run with `python3 -I`, and
`pdftotext` from poppler (`poppler-utils` on Debian and Ubuntu).

## Files

- `candidates.py`: the downloads, the ranking and the output.
- `appendix_a.py`: reads Appendix A from `pdftotext -bbox`'s word boxes.
- `test_appendix_a.py` and `fixtures/appendix-a.bbox.html`, a few rows of
  Appendix A (a US Government work): its test, run with
  `make candidates-test`.

## Removing it

When the table's input is compiled:

1. `git rm -r tools/compile-batches`, and delete its `cache/` if one is left.
2. Delete the "Temporary: compiling batches of rows" block at the end of the
   `Makefile` (`candidates`, `candidates-test` and `N`), and those two from its
   `.PHONY` line.
3. Delete the "Compiling a batch" section of `INSTRUCTIONS.md`, or keep the
   lessons in it that still apply to adding a row by hand.
