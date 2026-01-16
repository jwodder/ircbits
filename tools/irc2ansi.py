# TODO:
# - Don't append SGR0 if the current style is null/default
from __future__ import annotations
import argparse
import re
import sys
from unicodedata import category
import unittest


def irc2ansi(s: str) -> str:
    # Also escapes non-IRC-formatting control characters
    # Not supported: \x11 - Monospace
    s = s.replace("\x1b", "<1B>")
    s = re.sub(
        r"\x03(?:(?P<fg>[0-9]{1,2})(?:,(?P<bg>[0-9]{1,2}))?)?",
        lambda m: set_color(m["fg"], m["bg"]),
        s,
    )
    s = re.sub(
        r"\x03(?:(?P<fg>[A-Fa-f0-9]{6})(?:,(?P<bg>[A-Fa-f0-9]{6}))?)?",
        lambda m: set_hex_color(m["fg"], m["bg"]),
        s,
    )
    out = ""
    bold = False
    italic = False
    underline = False
    strikethrough = False
    reverse = False
    for c in s:
        match c:
            case "\x02":
                if bold:
                    out += "\x1b[22m"
                    bold = False
                else:
                    out += "\x1b[1m"
                    bold = True
            case "\x1d":
                if italic:
                    out += "\x1b[23m"
                    italic = False
                else:
                    out += "\x1b[3m"
                    italic = True
            case "\x1f":
                if underline:
                    out += "\x1b[24m"
                    underline = False
                else:
                    out += "\x1b[4m"
                    underline = True
            case "\x1e":
                if strikethrough:
                    out += "\x1b[29m"
                    strikethrough = False
                else:
                    out += "\x1b[9m"
                    strikethrough = True
            case "\x16":
                if reverse:
                    out += "\x1b[27m"
                    reverse = False
                else:
                    out += "\x1b[7m"
                    reverse = True
            case "\x0f":
                bold = False
                italic = False
                underline = False
                strikethrough = False
                reverse = False
                out += "\x1b[m"
            case c if category(c).startswith("C") and c != "\x1b":
                out += f"<{ord(c):02X}>"
            case c:
                out += c
    out += "\x1b[m"
    return out


def set_color(fg: str | None, bg: str | None) -> str:
    if fg is None:
        return "\x1b[39;49m"
    else:
        s = "\x1b["
        fgindex = int(fg)
        if fgindex == 99:
            s += "39"
        else:
            s += "38:5:" + str(IRC_INDEX_TO_ANSI_INDEX[fgindex])
        if bg is not None:
            s += ";"
            bgindex = int(bg)
            if bgindex == 99:
                s += "49"
            else:
                s += "48:5:" + str(IRC_INDEX_TO_ANSI_INDEX[bgindex])
        s += "m"
        return s


def set_hex_color(fg: str | None, bg: str | None) -> str:
    if fg is None:
        return "\x1b[39;49m"
    else:
        s = "\x1b["
        r, g, b = parse_hex_rgb(fg)
        s += f"38:2:{r}:{g}:{b}"
        if bg is not None:
            r, g, b = parse_hex_rgb(bg)
            s += f";48:2:{r}:{g}:{b}"
        s += "m"
        return s


def parse_hex_rgb(s: str) -> tuple[int, int, int]:
    r = int(s[:2], base=16)
    g = int(s[2:4], base=16)
    b = int(s[4:], base=16)
    return (r, g, b)


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
        ansi = "IRC \x1b[1mis \x1b[38:5:9;48:5:12mso \x1b[39;49mgreat\x1b[m!\x1b[m"
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
