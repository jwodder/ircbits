from __future__ import annotations
import argparse
from dataclasses import dataclass, field
import re
import sys
from unicodedata import category
import unittest

__all__ = ["irc2ansi"]

REGEX = re.compile(
    r"""
    (?P<bold>\x02)
    |(?P<color>\x03(?:(?P<fg>[0-9]{1,2})(?:,(?P<bg>[0-9]{1,2}))?)?)
    |(?P<hexcolor>\x04(?:(?P<fgx>[A-Fa-f0-9]{6})(?:,(?P<bgx>[A-Fa-f0-9]{6}))?)?)
    |(?P<reset>\x0f)
    |(?P<reverse>\x16)
    |(?P<italic>\x1d)
    |(?P<strikethrough>\x1e)
    |(?P<underline>\x1f)
    |(?P<char>.)
""",
    flags=re.X,
)


@dataclass(frozen=True)
class Color100:
    index: int

    def to_ansi(self, fg: bool) -> str:
        match (self.index, fg):
            case (99, True):
                return "39"
            case (99, False):
                return "49"
            case (index, True):
                return "38:5:" + str(IRC_INDEX_TO_ANSI_INDEX[index])
            case (index, False):
                return "48:5:" + str(IRC_INDEX_TO_ANSI_INDEX[index])
            case _:
                raise AssertionError("Unreachable")


@dataclass(frozen=True)
class RgbColor:
    r: int
    g: int
    b: int

    @classmethod
    def from_hex(cls, s: str) -> RgbColor:
        r = int(s[:2], base=16)
        g = int(s[2:4], base=16)
        b = int(s[4:], base=16)
        return RgbColor(r, g, b)

    def to_ansi(self, fg: bool) -> str:
        if fg:
            return f"38:2:{self.r}:{self.g}:{self.b}"
        else:
            return f"48:2:{self.r}:{self.g}:{self.b}"


DEFAULT_COLOR = Color100(99)


@dataclass
class Style:
    bold: bool = field(init=False, default=False)
    italic: bool = field(init=False, default=False)
    underline: bool = field(init=False, default=False)
    strikethrough: bool = field(init=False, default=False)
    reverse: bool = field(init=False, default=False)
    fg: Color100 | RgbColor = field(init=False, default=DEFAULT_COLOR)
    bg: Color100 | RgbColor = field(init=False, default=DEFAULT_COLOR)

    def __bool__(self) -> bool:
        return (
            self.bold
            or self.italic
            or self.underline
            or self.strikethrough
            or self.reverse
            or self.fg != DEFAULT_COLOR
            or self.bg != DEFAULT_COLOR
        )

    def toggle_bold(self) -> str:
        if self.bold:
            self.bold = False
            return "\x1b[22m"
        else:
            self.bold = True
            return "\x1b[1m"

    def toggle_italic(self) -> str:
        if self.italic:
            self.italic = False
            return "\x1b[23m"
        else:
            self.italic = True
            return "\x1b[3m"

    def toggle_underline(self) -> str:
        if self.underline:
            self.underline = False
            return "\x1b[24m"
        else:
            self.underline = True
            return "\x1b[4m"

    def toggle_strikethrough(self) -> str:
        if self.strikethrough:
            self.strikethrough = False
            return "\x1b[29m"
        else:
            self.strikethrough = True
            return "\x1b[9m"

    def toggle_reverse(self) -> str:
        if self.reverse:
            self.reverse = False
            return "\x1b[27m"
        else:
            self.reverse = True
            return "\x1b[7m"

    def reset(self) -> str:
        self.bold = False
        self.italic = False
        self.underline = False
        self.strikethrough = False
        self.reverse = False
        self.fg = DEFAULT_COLOR
        self.bg = DEFAULT_COLOR
        return "\x1b[m"

    def set_color(
        self, fg: Color100 | RgbColor | None, bg: Color100 | RgbColor | None
    ) -> str:
        if fg is None:
            fg = DEFAULT_COLOR
            bg = DEFAULT_COLOR
        self.fg = fg
        s = "\x1b[" + fg.to_ansi(True)
        if bg is not None:
            self.bg = bg
            s += ";" + bg.to_ansi(False)
        s += "m"
        return s


def irc2ansi(s: str) -> str:
    # Also escapes non-IRC-formatting control characters
    # Not supported: \x11 - Monospace
    out = ""
    pos = 0
    style = Style()
    while pos < len(s):
        m = REGEX.match(s, pos)
        assert m
        pos += m.end() - m.start()
        if m["bold"] is not None:
            out += style.toggle_bold()
        elif m["italic"] is not None:
            out += style.toggle_italic()
        elif m["underline"] is not None:
            out += style.toggle_underline()
        elif m["strikethrough"] is not None:
            out += style.toggle_strikethrough()
        elif m["reverse"] is not None:
            out += style.toggle_reverse()
        elif m["reset"] is not None:
            out += style.reset()
        elif m["color"] is not None:
            if (fgs := m["fg"]) is not None:
                fg = Color100(int(fgs))
            else:
                fg = None
            if (bgs := m["bg"]) is not None:
                bg = Color100(int(bgs))
            else:
                bg = None
            out += style.set_color(fg, bg)
        elif m["hexcolor"] is not None:
            if (fgs := m["fgx"]) is not None:
                fgx = RgbColor.from_hex(fgs)
            else:
                fgx = None
            if (bgs := m["bgx"]) is not None:
                bgx = RgbColor.from_hex(bgs)
            else:
                bgx = None
            out += style.set_color(fgx, bgx)
        elif m["char"] is not None:
            c = m["char"]
            if category(c).startswith("C"):
                out += f"<{ord(c):02X}>"
            else:
                out += c
        else:
            raise AssertionError("Unhandled construct")
    if style:
        out += style.reset()
    return out


IRC_INDEX_TO_ANSI_INDEX = [
    15,
    0,
    4,
    2,
    9,
    1,
    5,
    3,
    11,
    10,
    6,
    14,
    12,
    13,
    8,
    7,
    52,
    94,
    100,
    58,
    22,
    29,
    23,
    24,
    17,
    54,
    53,
    89,
    88,
    130,
    142,
    64,
    28,
    35,
    30,
    25,
    18,
    91,
    90,
    125,
    124,
    166,
    184,
    106,
    34,
    49,
    37,
    33,
    19,
    129,
    127,
    161,
    196,
    208,
    226,
    154,
    46,
    86,
    51,
    75,
    21,
    171,
    201,
    198,
    203,
    215,
    227,
    191,
    83,
    122,
    87,
    111,
    63,
    177,
    207,
    205,
    217,
    223,
    229,
    193,
    157,
    158,
    159,
    153,
    147,
    183,
    219,
    212,
    16,
    233,
    235,
    237,
    239,
    241,
    244,
    247,
    250,
    254,
    231,
]


class TestExamples(unittest.TestCase):
    # Examples taken from <https://modern.ircdocs.horse/formatting#examples>

    def test_example1(self) -> None:
        irc = "I love \x033IRC! \x03It is the \x037best protocol ever!"
        ansi = "I love \x1b[38:5:2mIRC! \x1b[39;49mIt is the \x1b[38:5:3mbest protocol ever!\x1b[m"
        self.assertEqual(irc2ansi(irc), ansi)

    def test_example2(self) -> None:
        irc = "This is a \x1d\x0313,9cool \x03message"
        ansi = "This is a \x1b[3m\x1b[38:5:13;48:5:10mcool \x1b[39;49mmessage\x1b[m"
        self.assertEqual(irc2ansi(irc), ansi)

    def test_example3(self) -> None:
        irc = "IRC \x02is \x034,12so \x03great\x0f!"
        ansi = "IRC \x1b[1mis \x1b[38:5:9;48:5:12mso \x1b[39;49mgreat\x1b[m!"
        self.assertEqual(irc2ansi(irc), ansi)

    def test_example4(self) -> None:
        irc = "Rules: Don't spam 5\x0313,8,6\x03,7,8, and especially not \x029\x02\x1d!"
        ansi = "Rules: Don't spam 5\x1b[38:5:13;48:5:11m,6\x1b[39;49m,7,8, and especially not \x1b[1m9\x1b[22m\x1b[3m!\x1b[m"
        self.assertEqual(irc2ansi(irc), ansi)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("-o", "--outfile", default="-")
    parser.add_argument("infile", nargs="*", default=["-"])
    args = parser.parse_args()
    if args.outfile == "-":
        outfp = sys.stdout
    else:
        outfp = open(args.outfile, "w", encoding="utf-8")
    with outfp:
        for fname in args.infile:
            if fname == "-":
                infp = sys.stdin
            else:
                infp = open(fname, "r", encoding="utf-8")
            with infp:
                for line in infp:
                    line = line.strip("\r\n")
                    print(irc2ansi(line), file=outfp)


if __name__ == "__main__":
    main()
