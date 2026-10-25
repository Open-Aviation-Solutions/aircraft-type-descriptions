# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# TEMPORARY: part of the tooling for compiling batches of rows (#8, #9). It is
# removed, with the rest of tools/compile-batches/, once the table's input is
# compiled; see README.md here for what removing it involves. Nothing in the
# crate, build.rs or `make check` uses it.

"""Prints the next designators to compile into the table, with the facts FAA
Order JO 7360.1 and CASA's aircraft register give for each.

The candidates are the designators on CASA's register, by number of aircraft,
highest first, ties by designator: those the order gives FAA weight class S or
S+, less its special (generic) designators and the designators already in the
table. Run it as `make candidates N=50`.

The current edition of the order and the register's data file are downloaded
into cache/ (git-ignored) the first time, and used from there until
`--refresh`. Each file's URL, date and SHA-256 are kept in cache/sources.json
and printed, so a batch can cite what it used.
"""

import argparse
import collections
import csv
import datetime
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tomllib
import urllib.error
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import appendix_a  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
CACHE = os.path.join(HERE, "cache")
MANIFEST = os.path.join(CACHE, "sources.json")
TABLE = os.path.join(HERE, "..", "..", "aircraft-type-descriptions.toml")

# The FAA's page for the order's current edition, which links its PDF.
ORDER_PAGE = (
    "https://www.faa.gov/regulations_policies/orders_notices/index.cfm/go/"
    "document.current/documentnumber/7360.1"
)
REGISTER_URL = "https://services.casa.gov.au/CSV/acrftreg.csv"
REGISTER_PAGE = (
    "https://www.casa.gov.au/aircraft/aircraft-registration/data-files-registered-aircraft"
)
REGISTER_FILE = os.path.join(CACHE, "acrftreg.csv")

# The register's Engtype, as the order's engine type letter (paragraph 2-10a).
# A diesel is a piston engine, and so is "Rotary" (a Wankel, such as the
# Austro AE50R), as the owner ruled on #11; a turboshaft is a turboprop, as in
# the table. "Not Applicable" has none, so never agrees.
REGISTER_ENGINE = {
    "Piston": "P",
    "Diesel Engine": "P",
    "Rotary": "P",
    "Turboprop": "T",
    "Turboshaft": "T",
    "Turbofan": "J",
    "Turbojet": "J",
    "Electric": "E",
}
REGISTER_GLIDER = {"Glider", "Motor-Glider"}

# The order's class and engine type as the table's airframe and engine-type.
AIRFRAME = {
    "Fixed-wing": "landplane",
    "@Fixed-wing": "amphibian",
    "$Fixed-wing": "seaplane",
    "Helicopter": "helicopter",
    "Gyroplane": "gyrocopter",
}
ENGINE_TYPE = {"P": "piston", "T": "turboprop", "J": "jet"}
WTC = {"Light": "light", "Medium": "medium", "Heavy": "heavy", "Super": "super"}


class Failure(Exception):
    pass


def now() -> str:
    return datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%d %H:%M UTC")


def fetch(url: str):
    try:
        with urllib.request.urlopen(url, timeout=120) as response:
            return response.read(), response.headers
    except urllib.error.HTTPError as e:
        raise Failure(f"{url} answered HTTP {e.code}") from e
    except urllib.error.URLError as e:
        raise Failure(f"{url} could not be fetched: {e.reason}") from e


def record(path: str, url: str, **facts) -> dict:
    with open(path, "rb") as f:
        data = f.read()
    return {
        "file": os.path.basename(path),
        "url": url,
        "bytes": len(data),
        "sha256": hashlib.sha256(data).hexdigest(),
        **facts,
    }


def load_manifest() -> dict:
    if os.path.exists(MANIFEST):
        with open(MANIFEST) as f:
            return json.load(f)
    return {}


def save_manifest(manifest: dict) -> None:
    with open(MANIFEST, "w") as f:
        json.dump(manifest, f, indent=2)
        f.write("\n")


def _dd(page: str, term: str) -> str | None:
    m = re.search(rf"<dt>{re.escape(term)}</dt>\s*<dd>(.*?)</dd>", page, re.S)
    return re.sub(r"\s+", " ", m.group(1)).strip() if m else None


def fetch_order() -> dict:
    page = fetch(ORDER_PAGE)[0].decode("utf-8", "replace")
    links = set(re.findall(r'href="(https://www\.faa\.gov/[^"]*7360\.1[^"]*\.pdf)"', page))
    if len(links) != 1:
        raise Failure(f"{ORDER_PAGE} does not link one PDF of the order: {sorted(links)}")
    url = links.pop()
    data, headers = fetch(url)
    if not data.startswith(b"%PDF"):
        raise Failure(f"{url} did not give a PDF")
    path = os.path.join(CACHE, os.path.basename(url))
    with open(path, "wb") as f:
        f.write(data)
    notices = []
    m = re.search(r"<dt>Current notices</dt>(.*?)</table>", page, re.S)
    for row in re.findall(r"<tr>(.*?)</tr>", m.group(1) if m else "", re.S):
        cells = [re.sub(r"<[^>]+>|\s+", " ", c).strip() for c in re.findall(r"<td>(.*?)</td>", row, re.S)]
        if len(cells) == 5:
            notices.append({"number": cells[0], "title": cells[2], "effective": cells[3], "cancelled": cells[4]})
    return record(
        path,
        url,
        number=_dd(page, "Number"),
        title=_dd(page, "Title"),
        issued=_dd(page, "Date issued"),
        page=ORDER_PAGE,
        notices=notices,
        retrieved=now(),
        last_modified=headers.get("Last-Modified"),
    )


BY_HAND = (
    "CASA's server refuses scripted downloads, so the register is normally "
    f"downloaded in a browser: get acrftreg.csv from {REGISTER_PAGE}, and give it "
    "with `make candidates REGISTER=path/to/acrftreg.csv REGISTER_DATE='<the "
    "date CASA gives for the file>'`."
)


def fetch_register() -> dict:
    try:
        data, headers = fetch(REGISTER_URL)
    except Failure as e:
        raise Failure(f"{e}.\n{BY_HAND}") from e
    if not data.lstrip(b"\xef\xbb\xbf").startswith(b"Mark,"):
        raise Failure(f"{REGISTER_URL} did not give the register's CSV file (a challenge page?).\n{BY_HAND}")
    with open(REGISTER_FILE, "wb") as f:
        f.write(data)
    return record(
        REGISTER_FILE,
        REGISTER_URL,
        retrieved=now(),
        last_modified=headers.get("Last-Modified"),
    )


def take_register(path: str, date: str | None) -> dict:
    """Takes a register file downloaded by hand into the cache.

    Its file time is when it was downloaded, not the file's edition, which
    only CASA's page (or the browser's download) gives: `date`, if given.
    """
    if not os.path.isfile(path):
        raise Failure(f"no register file at {path}")
    read_register(path)  # refuse anything but the register before taking it
    modified = datetime.datetime.fromtimestamp(os.path.getmtime(path), datetime.UTC)
    if os.path.abspath(path) != os.path.abspath(REGISTER_FILE):
        shutil.copyfile(path, REGISTER_FILE)
    return record(
        REGISTER_FILE,
        REGISTER_URL,
        retrieved=f"by hand, downloaded (file time) {modified:%Y-%m-%d %H:%M UTC}",
        last_modified=date,
    )


def read_register(path: str):
    with open(path, encoding="utf-8-sig", newline="") as f:
        rows = list(csv.DictReader(f))
    if not rows or not {"ICAOtypedesig", "engnum", "Engtype", "Airframe"} <= rows[0].keys():
        raise Failure(f"{path} is not CASA's register data file (no ICAOtypedesig, engnum, Engtype, Airframe)")
    by_designator = collections.defaultdict(list)
    for row in rows:
        by_designator[row["ICAOtypedesig"].strip()].append(row)
    return rows, by_designator


def read_order(path: str):
    try:
        bbox = subprocess.run(
            ["pdftotext", "-bbox", path, "-"], check=True, capture_output=True, text=True
        ).stdout
    except FileNotFoundError as e:
        raise Failure("pdftotext is not installed (it is in poppler-utils)") from e
    except subprocess.CalledProcessError as e:
        raise Failure(f"pdftotext failed on {path}: {e.stderr.strip()}") from e
    return {d.designator: d for d in appendix_a.parse(bbox)}


def table_designators(path: str) -> set[str]:
    with open(path, "rb") as f:
        return set(tomllib.load(f).get("designators", {}))


def candidates(order, register, in_table):
    found = []
    for designator, aircraft in register.items():
        entry = order.get(designator)
        if entry is None or entry.special or entry.weight_class not in ("S", "S+"):
            continue
        if designator in in_table:
            continue
        found.append((designator, entry, aircraft))
    found.sort(key=lambda c: (-len(c[2]), c[0]))
    return found


def as_row(entry, gliders: bool) -> str:
    airframe = "(airframe not from the order)" if gliders else AIRFRAME.get(entry.klass)
    engine_type = ENGINE_TYPE.get(entry.engine_type)
    if not airframe or not engine_type or entry.engine_count is None:
        return "(no value for it in the schema)"
    wtc = WTC.get(entry.wtc, "category left out")
    return f"{airframe}, {entry.engine_count} {engine_type}, {wtc}"


def describe(rank: int, designator: str, entry, aircraft) -> list[str]:
    differ = [
        a
        for a in aircraft
        if a["engnum"] != str(entry.engine_count)
        or REGISTER_ENGINE.get(a["Engtype"]) != entry.engine_type
    ]
    gliders = [a for a in aircraft if a["Airframe"] in REGISTER_GLIDER]
    gliders = gliders if entry.klass.endswith("Fixed-wing") else []
    agreement = f"all {len(aircraft)} agree" if not differ else f"{len(differ)} of {len(aircraft)} differ"
    lines = [
        f"{rank:3}  {designator:<5} {len(aircraft):5}  {agreement:<16}  "
        f"{entry.klass:<12} {entry.engines:<6} {entry.wtc or '(none)':<13} {as_row(entry, bool(gliders))}",
        "       " + "; ".join(entry.models),
    ]
    flags = []
    if entry.wtc not in WTC:
        flags.append(f"WTC {entry.wtc or 'not given'}: leave the category out, and say why in the notes")
    if differ:
        kinds = collections.Counter(f"{a['engnum']} {a['Engtype']}" for a in differ)
        listed = ", ".join(f"{n} with {k}" for k, n in kinds.most_common())
        flags.append(f"register disagrees on engines ({listed}): do not cite it")
    if gliders:
        flags.append(
            f"register files {len(gliders)} of {len(aircraft)} as gliders, the order as "
            f"{entry.klass}: take the airframe from a data sheet or the register, never the order"
        )
    lines += [f"       ! {flag}" for flag in flags]
    return lines


def print_sources(order: dict, register: dict, rows: int, designators: int) -> None:
    print(f"FAA Order {order['number']}, {order['title']} (issued {order['issued']})")
    print(f"  {order['url']}")
    print(f"  retrieved {order['retrieved']}; last modified {order['last_modified']}; {order['bytes']:,} bytes")
    print(f"  SHA-256 {order['sha256']}")
    for n in order["notices"]:
        print(f"  notice listed with it: {n['number']}, {n['title']} (effective {n['effective']}, cancelled {n['cancelled'] or '-'})")
    if order["notices"]:
        print("  ! A notice may add or change designators, and is not applied here: check it against the rows.")
    print("CASA, Australian civil aircraft register data file (acrftreg.csv)")
    print(f"  {register['url']}")
    edition = register["last_modified"] or "not known: take it from CASA's page, and give it as REGISTER_DATE="
    print(f"  edition (last modified) {edition}")
    print(
        f"  retrieved {register['retrieved']}; {register['bytes']:,} bytes; "
        f"{rows:,} aircraft under {designators:,} designators"
    )
    print(f"  SHA-256 {register['sha256']}")


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("-n", type=int, default=50, help="how many candidates to print (default 50)")
    parser.add_argument("--register", help="a copy of acrftreg.csv downloaded by hand, to use from now on")
    parser.add_argument("--register-date", help="the date CASA gives for the file given with --register")
    parser.add_argument("--refresh", action="store_true", help="download both files again")
    parser.add_argument("--table", default=TABLE, help="the data file whose rows are left out")
    args = parser.parse_args(argv)

    os.makedirs(CACHE, exist_ok=True)
    manifest = load_manifest()
    if args.refresh or "order" not in manifest or not os.path.exists(os.path.join(CACHE, manifest["order"]["file"])):
        manifest["order"] = fetch_order()
        save_manifest(manifest)
    if args.register:
        manifest["register"] = take_register(args.register, args.register_date)
        save_manifest(manifest)
    elif args.refresh or "register" not in manifest or not os.path.exists(REGISTER_FILE):
        manifest["register"] = fetch_register()
        save_manifest(manifest)

    order = read_order(os.path.join(CACHE, manifest["order"]["file"]))
    rows, register = read_register(REGISTER_FILE)
    found = candidates(order, register, table_designators(args.table))

    print_sources(manifest["order"], manifest["register"], len(rows), len(register))
    shown = found[: args.n]
    print()
    print(
        f"The next {len(shown)} of {len(found)} candidates: on the register, FAA weight class S or S+, "
        "not generic, not yet in the table; by aircraft, highest first."
    )
    print()
    print(f"{'#':>3}  {'Desig':<5} {'Count':>5}  {'Register engines':<16}  {'Class':<12} {'Engine':<6} {'WTC':<13} As a row")
    for rank, (designator, entry, aircraft) in enumerate(shown, start=1):
        print("\n".join(describe(rank, designator, entry, aircraft)))
    print()
    print(f"{len(found) - len(shown)} candidates left after these.")
    missing = sorted(((d, len(a)) for d, a in register.items() if d not in order), key=lambda m: (-m[1], m[0]))
    if missing:
        print()
        print(f"{len(missing)} designators on the register are not in the order, so are not candidates:")
        print("  " + ", ".join(f"{d or '(blank)'} ({n})" for d, n in missing))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (Failure, appendix_a.ParseError) as e:
        print(f"candidates: {e}", file=sys.stderr)
        sys.exit(1)
