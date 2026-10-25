# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# TEMPORARY: part of the tooling for compiling batches of rows (#8, #9). It is
# removed, with the rest of tools/compile-batches/, once the table's input is
# compiled; see README.md here for what removing it involves. Not part of
# `make check`: run it with `make candidates-test`.

"""Tests the parser of Appendix A on fixtures/appendix-a.bbox.html."""

import os
import re
import sys
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from appendix_a import Designator, ParseError, parse  # noqa: E402

with open(os.path.join(HERE, "fixtures", "appendix-a.bbox.html")) as f:
    FIXTURE = f.read()


class ParseTest(unittest.TestCase):
    def test_every_row_with_its_models(self):
        # A1's three models are centred on its row and A10's two either side
        # of it, so each is given to the row it is centred on.
        self.assertEqual(
            parse(FIXTURE),
            [
                Designator("A002", "Gyroplane", "1P/S", "Light", False, ["IRKUT, A-002"]),
                Designator(
                    "A1",
                    "Fixed-wing",
                    "1P/S+",
                    "Medium",
                    False,
                    ["DOUGLAS, A-1 Skyraider", "DOUGLAS, AD Skyraider", "DOUGLAS, EA-1 Skyraider"],
                ),
                Designator(
                    "A10",
                    "Fixed-wing",
                    "2J/L",
                    "Medium",
                    True,
                    ["FAIRCHILD (1), A-10 Thunderbolt 2", "FAIRCHILD (1), OA-10 Thunderbolt 2"],
                ),
                Designator(
                    "AT8T",
                    "Fixed-wing",
                    "1T/S+",
                    "Light/Medium",
                    False,
                    ["AIR TRACTOR, AT-802", "AIR TRACTOR, AT-802 Fire Boss"],
                ),
                Designator(
                    "AVTR",
                    "@Fixed-wing",
                    "1P/S",
                    "Light",
                    False,
                    ["AERO ADVENTURE, Aventura 2", "ARNET PEREYRA, Aventura 2"],
                ),
                Designator("AXE", "Fixed-wing", "8E/", "Light", False, ["SKYFLY, Axe"]),
                Designator(
                    "BABY",
                    "Helicopter",
                    "1P/S",
                    "Light",
                    False,
                    ["CANADIAN HOME ROTORS, Baby Belle", "CANADIAN HOME ROTORS, Safari"],
                ),
                Designator("BALL", "", None, None, False, ["(any manufacturer), Balloon"]),
                Designator(
                    "BARC", "Fixed-wing", "1P/S", "Light", False, ["BUETHE, Barracuda", "JEFFAIR, Barracuda"]
                ),
            ],
        )

    def test_engines_and_weight_class(self):
        rows = {d.designator: d for d in parse(FIXTURE)}
        at8t, axe, ball = rows["AT8T"], rows["AXE"], rows["BALL"]
        self.assertEqual((at8t.engine_count, at8t.engine_type, at8t.weight_class), (1, "T", "S+"))
        # The weight class is blank for some types.
        self.assertEqual((axe.engine_count, axe.engine_type, axe.weight_class), (8, "E", None))
        self.assertTrue(ball.special)
        self.assertEqual((ball.engine_count, ball.engine_type, ball.weight_class), (None, None, None))

    def test_refuses_a_model_line_it_cannot_place(self):
        # Without BARC's row (the words on its line), its two model lines are
        # centred on no row.
        without_barc = re.sub(r'\s*<word [^>]*yMin="645\.932500"[^>]*>[^<]*</word>', "", FIXTURE)
        with self.assertRaisesRegex(ParseError, "BUETHE, Barracuda"):
            parse(without_barc)


if __name__ == "__main__":
    unittest.main()
