[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.91-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ircbits.svg)](https://opensource.org/licenses/MIT)

`listchans` logs into an IRC network, sends a `LIST` command, outputs the
response as JSON, and disconnects.  That's all it does.

Options
=======

- `-c FILE`, `--config FILE` — Read IRC network connection details from the
  given configuration file, which must follow [the common `ircbits` config
  format][cfg] [default: `ircbits.toml`]

- `-o PATH`, `--outfile PATH` — Write the output to the given path.  By
  default, output is written to the standard output.

- `-P NAME`, `--profile NAME` — Use the connection details under the given
  profile in the configuration file [default: `irc`]

- `--trace` - Emit log events for every message sent & received

[cfg]: https://github.com/jwodder/ircbits/blob/master/doc/config-file.md

Output Format
=============

The output is a JSON array of the channels reported by the server, each given
as an object with the following fields:

- `"channel"` — the name of the channel
- `"clients"` — the number of users currently in the channel
- `"topic"` — the channel's topic
