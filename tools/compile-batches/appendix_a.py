# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# TEMPORARY: part of the tooling for compiling batches of rows (#8, #9). It is
# removed, with the rest of tools/compile-batches/, once the table's input is
# compiled; see README.md here for what removing it involves. Nothing in the
# crate, build.rs or `make check` uses it.

"""Parses Appendix A of FAA Order JO 7360.1 ("Decode - Aircraft Type
Designator") from the word boxes `pdftotext -bbox` gives for the order's PDF.

Each row of the appendix's table is a designator with its class, engines and
FAA weight class, ICAO wake turbulence category and other columns, and a cell
of manufacturer-and-model lines. The models cell can have many lines, and the
other cells are vertically centred on it, so each model line is given to the
designator whose row it is centred on: the lines from the topmost one not yet
taken down to as far below the designator as the topmost is above it.
"""

import html
import re
from dataclasses import dataclass, field

_WORD = re.compile(
    r'<word xMin="([\d.]+)" yMin="([\d.]+)" xMax="([\d.]+)" yMax="([\d.]+)">(.*?)</word>'
)
_DESIGNATOR = re.compile(r"^([A-Z0-9]{2,4})(\*?)$")
_CLASS = re.compile(r"^([@$]?)(Fixed-wing|Gyroplane|Helicopter|Powered-lift)$")
# Engine number and type / FAA weight class (paragraph 2-10), such as 2T/S+.
# The number is C for two engines coupled to one propeller; the weight class
# is left blank for some types.
_ENGINES = re.compile(r"^(\d+|C)([CEJPRT])/(J|H|L|S\+|S|)$")
_WTC = {"Light", "Medium", "Heavy", "Super", "Light/Medium", "Medium/Heavy"}
_HEADER_WORDS = {"Class", "Weight", "Designator", "WTC"}
# Words on one line differ in yMin by far less than this; lines by about 8.8.
_SAME_LINE = 1.5


class ParseError(Exception):
    pass


@dataclass
class Designator:
    designator: str
    # The order's own columns, as printed. `klass` is empty, and the rest are
    # None, for a special designator (paragraph 2-4), such as GLID.
    klass: str
    engines: str | None
    wtc: str | None
    # An asterisk on the designator: a single-piloted military turbojet (2-1a).
    single_piloted_military: bool = False
    models: list[str] = field(default_factory=list)

    @property
    def special(self) -> bool:
        return not self.klass

    def _engines(self):
        m = _ENGINES.match(self.engines or "")
        return m.groups() if m else (None, None, None)

    @property
    def engine_count(self) -> int | None:
        number = self._engines()[0]
        return int(number) if number and number.isdigit() else None

    @property
    def engine_type(self) -> str | None:
        """The letter of paragraph 2-10a: E, J, P, R or T. Its C, two engines
        coupled to one propeller, stands in place of the number (CT/S+), and
        then `engine_count` is None."""
        return self._engines()[1]

    @property
    def weight_class(self) -> str | None:
        """The FAA weight class (2-10b): J, H, L, S or S+; None if blank."""
        return self._engines()[2] or None


def _words(page: str):
    return [
        (float(x0), float(y0), float(x1), float(y1), html.unescape(text))
        for x0, y0, x1, y1, text in _WORD.findall(page)
    ]


def _lines(words):
    """Groups words into lines by yMin, each line's words in x order."""
    lines = []
    for word in sorted(words, key=lambda w: (w[1], w[0])):
        if lines and word[1] - lines[-1][0][1] < _SAME_LINE:
            lines[-1].append(word)
        else:
            lines.append([word])
    return [sorted(line) for line in lines]


def _is_appendix_a(words) -> bool:
    running_header = " ".join(w[4] for w in sorted(words, key=lambda w: (w[1], w[0])) if w[1] < 70)
    return running_header.endswith("Appendix A")


def _parse_page(words, number: int) -> list[Designator]:
    manufacturer = [w for w in words if w[4] == "MANUFACTURER,"]
    if len(manufacturer) != 1:
        raise ParseError(f"page {number}: no single MANUFACTURER, column header")
    models_x = manufacturer[0][0] - 3
    header_bottom = max(
        w[3] for w in words if w[4] in _HEADER_WORDS and w[1] < manufacturer[0][3] + 25
    )
    footer = [w[1] for w in words if w[1] > 700 and re.match(r"^A-\d+$", w[4])]
    body_bottom = (min(footer) if footer else 792) - 3
    body = [w for w in words if header_bottom < w[1] < body_bottom]

    rows = []
    for line in _lines([w for w in body if w[2] < models_x]):
        rows.append(_row(line, number))
    model_lines = [
        ((line[0][1] + line[0][3]) / 2, " ".join(w[4] for w in line))
        for line in _lines([w for w in body if w[0] >= models_x])
    ]

    taken = 0
    for i, (centre, designator) in enumerate(rows):
        above = centre if taken >= len(model_lines) else model_lines[taken][0]
        if above > centre + _SAME_LINE:
            above = centre  # the next line is below: this row has none above
        elif i > 0 and above <= rows[i - 1][0]:
            raise ParseError(
                f"page {number}: model line {model_lines[taken][1]!r} is not "
                f"centred on {designator.designator} or the row before it"
            )
        reach = centre + (centre - above) + _SAME_LINE
        while taken < len(model_lines) and model_lines[taken][0] <= reach:
            designator.models.append(model_lines[taken][1])
            taken += 1
        if not designator.models:
            raise ParseError(f"page {number}: {designator.designator} has no models")
    if taken != len(model_lines):
        raise ParseError(
            f"page {number}: model line {model_lines[taken][1]!r} is below the last row"
        )
    return [designator for _, designator in rows]


def _row(line, number: int):
    tokens = [w[4] for w in line]
    centre = (line[0][1] + line[0][3]) / 2
    m = _DESIGNATOR.match(tokens[0])
    if not m:
        raise ParseError(f"page {number}: not a designator row: {tokens}")
    designator = Designator(m.group(1), "", None, None, m.group(2) == "*")
    if len(tokens) == 1:
        return centre, designator  # a special designator, such as GLID
    if len(tokens) < 3 or not _CLASS.match(tokens[1]) or not _ENGINES.match(tokens[2]):
        raise ParseError(f"page {number}: cannot read the row {tokens}")
    designator.klass = tokens[1]
    designator.engines = tokens[2]
    if len(tokens) > 3 and tokens[3] in _WTC:
        designator.wtc = tokens[3]
    return centre, designator


def parse(bbox_html: str) -> list[Designator]:
    """Every designator in Appendix A, in the order's order.

    `bbox_html` is the output of `pdftotext -bbox` for the order (the whole of
    it, or any pages including the appendix); pages other than Appendix A's are
    passed over. Raises ParseError where the table is not as expected, rather
    than guess.
    """
    designators = []
    for number, page in enumerate(bbox_html.split("<page ")[1:], start=1):
        words = _words(page)
        if _is_appendix_a(words):
            designators.extend(_parse_page(words, number))
    if not designators:
        raise ParseError("no page of Appendix A found")
    seen = set()
    for d in designators:
        if d.designator in seen:
            raise ParseError(f"{d.designator} is listed twice")
        seen.add(d.designator)
    return designators
