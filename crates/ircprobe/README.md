[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.91-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ircbits.svg)](https://opensource.org/licenses/MIT)

`ircprobe` logs into an IRC network, fetches various details about the remote
server and/or network, outputs the results as JSON, and disconnects.

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

The output is a JSON object with the following fields:

- `"capabilities"` (object or `null`) — If the server supports capability
  negotiation, this field contains its supported capabilities as an object
  mapping capability names to capability values (or to `null` if no value is
  given); otherwise, this field is `null`

- `"server"` (object) — Details about the IRC server as given in the
  `RPL_MYINFO` reply.  This object contains the following fields:

    - `"name"` (string) — the name of the server
    - `"version"` (string) — the server's software version
    - `"user_modes"` (string) — the available user modes
    - `"channel_modes"` (string) — the available channel modes that do not take
      parameters
    - `"param_channel_modes"` (string or `null`) — the available channel modes
      that take parameters, or `null` if there are none

- `"isupport"` (object) — Features advertised by the server via the parameters
  of the `RPL_ISUPPORT` reply.  Each key of the object is a feature parameter
  name, and the corresponding value is the given parameter value (if any),
  `true` (if no value was given), or `false` (if no value was given and the
  parameter was negated).

- `"lusers"` (object or `null`) — Server statistics about users returned in
  response to a `LUSERS` command, or `null` if no information was returned.
  When present, this object contains the following fields:

    - `"operators"` (integer or `null`) — the number of IRC operators connected
      to the server, or `null` if not given
    - `"unknown_connections"` (integer or `null`) — the number of connections
      to the server that are currently in an unknown state, or `null` if not
      given
    - `"channels"` (integer or `null`) — the number of channels that currently
      exist on the server, or `null` if not given
    - `"local_clients"` (integer or `null`) — the number of clients currently
      directly connected to this server, or `null` if not given
    - `"max_local_clients"` (integer or `null`) — the maximum number of clients
      ever directly connected to this server at one time, or `null` if not
      given
    - `"global_clients"` (integer or `null`) — the number of clients currently
      globally connected to this server, or `null` if not given
    - `"max_global_clients"` (integer or `null`) — the maximum number of
      clients ever globally connected to this server at one time, or `null` if
      not given
    - `"luserclient_msg"` (string or `null`) — the message given in the
      `RPL_LUSERCLIENT` reply, or `None` if the reply was not sent
    - `"luserme_msg"` (string or `null`) — the message given in the
      `RPL_LUSERME` reply, or `None` if the reply was not sent
    - `"statsconn_msg"` (string or `null`) — the message given in the
      `RPL_STATSCONN` reply, or `None` if the reply was not sent

- `"motd"` (string or `null`) — the server's message of the day, or `null` if
  no MOTD is set

- `"version"` (object or `null`) — Information returned in a `RPL_VERSION`
  reply to a `VERSION` command, or `null` if no such reply was received.  When
  present, this object contains the following fields:

    - `"version"` (string)
    - `"server"` (string)
    - `"comments"` (string or `null`)

- `"admin"` (object or `null`) — Information returned in response to an `ADMIN`
  command, or `null` if no such response was received.  When present, this
  object contains the following fields:

    - `"loc1"` (string)
    - `"loc2"` (string)
    - `"email"` (string)

- `"links"` (list of objects or `null`) — Information returned in response to a
  `LINKS` command, or `null` if no such response was received.  When present,
  each object corresponds to a single `RPL_LINKS` reply and contains the
  following fields:

    - `"server1"` (string)
    - `"server2"` (string)
    - `"hopcount"` (integer)
    - `"server_info"` (string)

- `"info"` (list of strings) — Information returned in response to an `INFO`
  command.  Each string is the message parameter of a single `RPL_INFO` reply.
