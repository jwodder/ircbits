#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

# TODO:
# - Specially mark messages from servers?
# - Add an option to convert IRC formatting to ANSI escapes?
# - Show TAGMSG events?

from __future__ import annotations
import argparse
from collections.abc import Iterator
from dataclasses import dataclass, field
from datetime import datetime
import gzip
import json
from pathlib import Path
import re
from types import TracebackType
from typing import IO


@dataclass
class AutoFileDict:
    basedir: Path
    # `None` values represent files that were previously opened and then closed
    # on reaching a "disconnect" event; when they are next opened, it should be
    # in append mode.
    files: dict[str, IO[str] | None] = field(init=False, default_factory=dict)

    def __enter__(self) -> AutoFileDict:
        return self

    def __exit__(
        self,
        _exc_type: type[BaseException] | None,
        _exc_val: BaseException | None,
        _exc_tb: TracebackType | None,
    ) -> None:
        for fp in self.files.values():
            if fp is not None:
                fp.close()

    def __getitem__(self, fname: str) -> IO[str]:
        if fname not in self.files:
            self.files[fname] = (self.basedir / fname).open("w", encoding="utf-8")
        elif self.files[fname] is None:
            self.files[fname] = (self.basedir / fname).open("a", encoding="utf-8")
        fp = self.files[fname]
        assert fp is not None
        return fp

    def for_channel(self, channel: str) -> IO[str]:
        logname = channel[1:].replace("#", "_") + ".txt"
        return self[logname]

    def fileiter(self) -> Iterator[IO[str]]:
        return (fp for fp in self.files.values() if fp is not None)

    def clear(self) -> None:
        for k, fp in self.files.items():
            if fp is not None:
                fp.close()
                self.files[k] = None

    def close_channel(self, channel: str) -> None:
        logname = channel[1:].replace("#", "_") + ".txt"
        if (fp := self.files.get(logname)) is not None:
            fp.close()
            self.files[logname] = None


def main() -> None:
    parser = argparse.ArgumentParser(
        description=(
            "This script takes a series of logfiles created by `ircevents` and"
            " creates a directory of human-readable message logs, one file per"
            " channel."
        )
    )
    parser.add_argument("-o", "--outdir", type=Path, default="fmtlogs")
    parser.add_argument(
        "-S",
        "--system-logfile",
        type=Path,
        help="File in which to log non-channel-specific messages [default: {outdir}/SYSTEM.txt]",
    )
    parser.add_argument("infile", type=Path, nargs="*")
    args = parser.parse_args()
    args.outdir.mkdir(parents=True, exist_ok=True)
    if args.system_logfile is not None:
        sysfilepath = args.system_logfile
        sysfilepath.parent.mkdir(parents=True, exist_ok=True)
    else:
        sysfilepath = args.outdir / "SYSTEM.txt"
    my_nick: str | None = None
    with AutoFileDict(args.outdir) as files, sysfilepath.open(
        "w", encoding="utf-8"
    ) as sysfp:
        for fpath in args.infile:
            if fpath.suffix.lower() == ".gz":
                fp = gzip.open(fpath, "rt", encoding="utf-8")
            else:
                fp = fpath.open("r", encoding="utf-8")
            with fp:
                for line in fp:
                    data = json.loads(line)
                    dt = datetime.fromisoformat(data["timestamp"]).isoformat(
                        sep=" ", timespec="seconds"
                    )
                    if (src := data.get("source")) is not None:
                        if (nick := src.get("nickname")) is not None:
                            source = nick
                        else:
                            source = src["host"]
                    else:
                        source = None
                    if data["event"] in ("privmsg", "notice"):
                        assert source is not None
                        for t in data["targets"]:
                            if t.startswith("#") or t == my_nick:
                                s = f"[{dt}] "
                                if data["event"] == "notice":
                                    s += "[NOTICE] "
                                if m := re.fullmatch(
                                    r"\x01ACTION (.+)\x01", data["text"]
                                ):
                                    s += "* "
                                    msg = m[1]
                                else:
                                    msg = data["text"]
                                s += f"<{source}> {msg}"
                                if t.startswith("#"):
                                    ff = files.for_channel(t)
                                else:
                                    ff = sysfp
                                print(s, file=ff)
                    elif data["event"] == "topic":
                        channel = data["channel"]
                        if (nick := data["source"].get("nickname")) is not None:
                            source = nick
                        else:
                            source = data["source"]["host"]
                        if (topic := data["topic"]) is not None:
                            print(
                                f"[{dt}] # {source} changed the channel topic: {topic}",
                                file=files.for_channel(channel),
                            )
                        else:
                            print(
                                f"[{dt}] # {source} unset the channel topic",
                                file=files.for_channel(channel),
                            )
                    elif data["event"] == "joined":
                        channel = data["channel"]
                        print(
                            f"[{dt}] --- Joined {channel} ---",
                            file=files.for_channel(channel),
                        )
                    elif data["event"] == "connected":
                        print(f"[{dt}] --- Connected ---", file=sysfp)
                        my_nick = data["my_nick"]
                    elif data["event"] == "disconnected":
                        print(f"[{dt}] --- Disconnected ---", file=sysfp)
                        for ff in files.fileiter():
                            print(f"[{dt}] --- Disconnected ---", file=ff)
                        files.clear()
                    elif data["event"] == "kick" and my_nick in data["users"]:
                        assert source is not None
                        s = f"[{dt}] # {source} kicked you from {data['channel']}"
                        if (c := data["comment"]) is not None:
                            s += f": {c}"
                        print(s, file=files.for_channel(data["channel"]))
                        files.close_channel(data["channel"])
                    elif data["event"] == "invite" and data["nickname"] == my_nick:
                        assert source is not None
                        print(
                            f"[{dt}] # {source} invited you to {data['channel']}",
                            file=sysfp,
                        )
                    elif data["event"] == "mode" and data["target"] == my_nick:
                        assert source is not None
                        s = f"[{dt}] # {source} changed your mode: {data['modestring']}"
                        s += " ".join(data["arguments"])
                        print(s, file=sysfp)
                    elif data["event"] == "wallops":
                        assert source is not None
                        print(f"[{dt}] [WALLOPS] <{source}> {data['text']}", file=sysfp)
                    elif data["event"] == "error":
                        print(f"[{dt}] [ERROR] {data['reason']}", file=sysfp)


if __name__ == "__main__":
    main()
