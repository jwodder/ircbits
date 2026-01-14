[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.91-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ircbits.svg)](https://opensource.org/licenses/MIT)

`ircwatch` logs into an IRC network, joins a given set of channels, and then
pretty-prints all messages received to standard output until you hit Ctrl-C.

The channels to join can be given as arguments on the command line.  If no
channels are specified on the command line, `ircwatch` takes the list of
channels from the `<profile>.ircwatch.default-channels` key in the
configuration file (where `<profile>` is the name of the profile in use).

Options
=======

- `-c FILE`, `--config FILE` — Read IRC network connection details from the
  given configuration file, which must follow [the common `ircbits` config
  format][cfg] [default: `ircbits.toml`]

- `-P NAME`, `--profile NAME` — Use the connection details under the given
  profile in the configuration file [default: `irc`]

- `--trace` - Emit log events for every message sent & received

[cfg]: https://github.com/jwodder/ircbits/blob/master/doc/config-file.md
