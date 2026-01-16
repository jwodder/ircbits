#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

# TODO:
# - Specially mark messages from servers?

from __future__ import annotations
import argparse
from collections import deque
from collections.abc import Iterator
from datetime import UTC, datetime
import json
import os
import re
import sys
import time
from typing import IO
from zoneinfo import ZoneInfo
from irc2ansi import irc2ansi


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("-n", "--lines", type=int, default=10)
    parser.add_argument("-N", "--nickname")
    parser.add_argument(
        "-t",
        "--timezone",
        help="Convert log timestamps to the given timezone",
        metavar="IANA-TIMEZONE",
    )
    parser.add_argument("logfile")
    args = parser.parse_args()
    if args.timezone is not None:
        tz = ZoneInfo(args.timezone)
    else:
        tz = None
    tail: deque[str] = deque(maxlen=max(args.lines, 0))
    lineiter = follow(args.logfile)
    for line in lineiter:
        if line is not None:
            if (s := fmtevent(line, tz=tz, my_nick=args.nickname)) is not None:
                tail.append(s)
        else:
            for s in tail:
                print(s)
            break
    for line in lineiter:
        assert line is not None
        if (s := fmtevent(line, tz=tz, my_nick=args.nickname)) is not None:
            print(s)


def fmtevent(line: str, *, tz: ZoneInfo | None, my_nick: str | None) -> str | None:
    data = json.loads(line)
    dt = datetime.fromisoformat(data["timestamp"])
    if tz is not None:
        dt = dt.astimezone(tz)
    ts = dt.isoformat(sep=" ", timespec="seconds")
    if (src := data.get("source")) is not None:
        if (nick := src.get("nickname")) is not None:
            source = nick
        else:
            source = src["host"]
    else:
        source = None
    if data["event"] in ("privmsg", "notice"):
        assert source is not None
        if my_nick in data["targets"]:
            s = f"[{ts}] "
            if data["event"] == "notice":
                s += "[NOTICE] "
            else:
                s += "[PRIVMSG] "
            if m := re.fullmatch(r"\x01ACTION (.+)\x01", data["text"]):
                s += "* "
                msg = m[1]
            else:
                msg = data["text"]
            s += f"<{highlight(source)}> {irc2ansi(msg)}"
            return s
        else:
            target = ",".join(
                highlight(t) for t in data["targets"] if t.startswith("#")
            )
            if not target:
                return None
            s = f"[{ts}] [{target}] "
            if data["event"] == "notice":
                s += "[NOTICE] "
            if m := re.fullmatch(r"\x01ACTION (.+)\x01", data["text"]):
                s += "* "
                msg = m[1]
            else:
                msg = data["text"]
            if (nick := data["source"].get("nickname")) is not None:
                source = nick
            else:
                source = data["source"]["host"]
            s += f"<{highlight(source)}> {irc2ansi(msg)}"
            return s
    elif data["event"] == "topic":
        assert source is not None
        channel = data["channel"]
        if (topic := data["topic"]) is not None:
            return f"[{ts}] # {highlight(source)} changed the {highlight(channel)} topic: {irc2ansi(topic)}"
        else:
            return f"[{ts}] # {highlight(source)} unset the {highlight(channel)} topic"
    elif data["event"] == "connected":
        return f"[{ts}] # Connected"
    elif data["event"] == "disconnected":
        return f"[{ts}] # Disconnected"
    elif data["event"] == "kick" and my_nick in data["users"]:
        assert source is not None
        s = f"[{ts}] # {highlight(source)} kicked you from {highlight(data['channel'])}"
        if (c := data["comment"]) is not None:
            s += f": {irc2ansi(c)}"
        return s
    elif data["event"] == "invite" and data["nickname"] == my_nick:
        assert source is not None
        return (
            f"[{ts}] # {highlight(source)} invited you to {highlight(data['channel'])}"
        )
    elif data["event"] == "mode" and data["target"] == my_nick:
        assert source is not None
        s = f"[{ts}] # {highlight(source)} changed your mode: {data['modestring']}"
        s += " ".join(data["arguments"])
        return s
    elif data["event"] == "wallops":
        assert source is not None
        return f"[{ts}] [WALLOPS] <{highlight(source)}> {irc2ansi(data['text'])}"
    elif data["event"] == "error":
        return f"[{ts}] [ERROR] {data['reason']}"
    else:
        return None


# Based on <https://www.dabeaz.com/generators/Generators.pdf>
# Cf. <https://stackoverflow.com/questions/12523044/>
def follow(fname: str) -> Iterator[str | None]:
    i = inode(fname)
    hit_eof = False
    while True:
        with open(fname, "r", encoding="utf-8") as fp:
            while True:
                line = fp.readline()
                if not line:
                    if not hit_eof:
                        yield None
                        hit_eof = True
                    try:
                        time.sleep(0.1)
                    except KeyboardInterrupt:
                        return
                    if inode(fname) != i:
                        break
                    else:
                        continue
                yield line


def inode(path: str) -> int:
    return os.stat(path).st_ino


colors = set(range(1, 15))
colors.update(range(26, 52))
colors.update(range(58, 232))
colors.remove(7)
colors.remove(58)
colors.remove(59)
colors.remove(60)
colors.remove(61)
colors.remove(62)
colors.remove(88)
colors.remove(89)
colors.remove(90)
colors.remove(91)
colors.remove(92)
colors.remove(93)
colors.remove(124)
COLORS = sorted(colors)


def highlight(s: str) -> str:
    # <https://github.com/osa1/tiny/blob/07b0ceb6b0fe92146f28f2bde03938daad422ad0/crates/libtiny_tui/src/messaging.rs#L360>
    # Note that we can't use Python's hash() function, as that's randomized on
    # each program invocation.
    h = 5381
    for ch in s:
        h = h * 33 + ord(ch)
    index = COLORS[h % len(COLORS)]
    return f"\x1b[38:5:{index}m{s}\x1b[m"


if __name__ == "__main__":
    main()
