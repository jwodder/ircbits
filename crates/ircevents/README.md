[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.91-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ircbits.svg)](https://opensource.org/licenses/MIT)

`ircevents` logs into an IRC network, joins a given set of channels, and then
runs indefinitely, outputting a JSON object for each message & event that
occurs.

A running `ircevents` instance can be shut down gracefully by sending it SIGINT
(Ctrl-C) or SIGTERM, which will cause it to send a `QUIT` message to the server
and then exit once the server closes the connection.

Options
=======

- `-c FILE`, `--config FILE` — Read IRC network connection details from the
  given configuration file, which must follow [the common `ircbits` config
  format][cfg] [default: `ircbits.toml`]

- `-o PATH`, `--outfile PATH` — Append the output to the given path [default:
  `ircevents.jsonl`]

- `-P NAME`, `--profile NAME` — Use the connection details under the given
  profile in the configuration file [default: `irc`]

- `-R SIZE`, `--rotate-size SIZE` — Rotate the output file if it would exceed
  the given size.  `SIZE` can be given as an integer number of bytes or as a
  floating-point number followed by a unit like `kB` or `MiB`.

  When the output file is rotated, the old file is moved to
  `{dirpath}/{basename}.{timestamp}.{ext}`, where `{dirpath}`, `{basename}`,
  and `{ext}` are taken from the original output file path and `{timestamp}` is
  the current timestamp in the format `%Y%m%dT%H%M%SZ`.  (If the output file
  path does not have an extension, the "`.{ext}`" component is omitted.)

- `--trace` - Emit log events for every message sent & received

[cfg]: https://github.com/jwodder/ircbits/blob/master/doc/config-file.md

Configuration
=============

In addition to connection details, `ircevents` reads further configuration from
the `ircevents` subtable of the selected profile table in the configuration
file.  This subtable contains the following fields:

- `away` (string; optional) — If this is set, the given string will be sent as
  the argument of an `AWAY` message immediately after logging in

- `channels` (nonempty list of strings; required) — the names of the channels
  to join

Output Format
=============

`ircevents`'s output is in JSON Lines format, with each line representing an
IRC message or event as a JSON object.  Every such object contains at least the
following fields:

- `"timestamp"` — the time at which the event occurred as an RFC 3339 timestamp

- `"source"` — The source of a received message, or `null` if the message was
  received without a source.  When non-`null`, `"source"` is an either an
  object that contains a single `"host"` string field (when the source is a
  host or server) or an object containing `"nickname"` (string), `"user"`
  (string or `null`), and `"host"` (string or `null`) fields (when the source
  is a user on the network).

  This field is not present for the `"connected"`, `"joined"`, `"parse_error"`,
  and `"disconnected"` events.

- `"event"` — the type of event that occurred; see below

Further fields depend on the type of event as detailed below.  For client
message events, the additional fields are the (possibly parsed) parameters of
the message; see <https://modern.ircdocs.horse> for their semantics.

`"admin"` Event
----------------

Emitted when an `ADMIN` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional field:

- `"target"` (string or `null`)

`"authenticate"` Event
----------------

Emitted when an `AUTHENTICATE` message is received.

This event should not be emitted during normal operation (as it is handled by
the login procedure) and is included only for completeness.

JSON objects for this event type have the following additional field:

- `"parameter"` (string)

`"away"` Event
----------------

Emitted when an `AWAY` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional field:

- `"text"` (string or `null`)

`"cap"` Event
----------------

Emitted when a `CAP` message is received outside of the login procedure.

JSON objects for this event type have the following additional field:

- `"parameters"` (list of strings)

`"connect"` Event
----------------

Emitted when a `CONNECT` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional fields:

- `"target_server"` (string)
- `"port"` (integer or `null`)
- `"remote_server"` (string or `null`)

`"connected"` Event
-------------------

Emitted when the program successfully finishes connecting to & logging in to an
IRC network.

JSON objects for this event type have the following additional fields:

- `"capabilities"` (object or `null`) — If the server supports capability
  negotiation, this field contains its supported capabilities as an object
  mapping capability names to capability values (or to `null` if no value is
  given); otherwise, this field is `null`

- `"my_nick"` (string) — The nickname with which the program logged into IRC as
  given in the `RPL_WELCOME` reply

- `"welcome_msg"` (string) — The message given in the `RPL_WELCOME` reply

- `"yourhost_msg"` (string) — The message given in the `RPL_YOURHOST` reply

- `"created_msg"` (string) — The message given in the `RPL_CREATED` reply

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

- `"lusers"` (object) — Server statistics about users as supplied in the
  response to an optional implicit `LUSERS` command upon connection.  This
  object contains the following fields:

    - `"operators"` (integer or `null`) — the number of IRC operators connected
      to the server, or `null` if not given
    - `"unknown_connections"` (integer or `null`) the number of connections to
      the server that are currently in an unknown state, or `null` if not given
    - `"channels"` (integer or `null`) the number of channels that currently
      exist on the server, or `null` if not given
    - `"local_clients"` (integer or `null`) the number of clients currently
      directly connected to this server, or `null` if not given
    - `"max_local_clients"` (integer or `null`) the maximum number of clients
      ever directly connected to this server at one time, or `null` if not
      given
    - `"global_clients"` (integer or `null`) the number of clients currently
      globally connected to this server, or `null` if not given
    - `"max_global_clients"` (integer or `null`) the maximum number of clients
      ever globally connected to this server at one time, or `null` if not
      given

- `"motd"` (string or `null`) — the server's message of the day, or `null` if
  no MOTD is set

- `"mode"` (string or `null`) — the user's client modes, or `null` if the
  server did not report the mode upon login

`"disconnected"` Event
----------------------

Emitted when the connection to the IRC network closes.

JSON objects for this event type have no additional fields.

`"error"` Event
----------------

Emitted when a `ERROR` message is received.

JSON objects for this event type have the following additional field:

- `"reason"` (string)

`"help"` Event
----------------

Emitted when a `HELP` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional field:

- `"subject"` (string or `null`)

`"info"` Event
----------------

Emitted when an `INFO` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have no additional fields.

`"invite"` Event
----------------

Emitted when a `INVITE` message is received.

JSON objects for this event type have the following additional fields:

- `"nickname"` (string)
- `"channel"` (string)

`"join"` Event
----------------

Emitted when a `JOIN` message is received.

JSON objects for this event type have the following additional field:

- `"channels"` (list of strings)

`"join0"` Event
----------------

Emitted when a `JOIN` message with a parameter of `0` is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have no additional fields.

`"joined"` Event
----------------

Emitted when the program successfully finishes joining a channel.

JSON objects for this event type have the following additional fields:

- `"channel"` (string) — the name of the channel as given in `RPL_NAMREPLY`
  messages
- `"topic"` (string or `null`) — the channel's topic, or `null` if no topic is
  set
- `"topic_set_by"` (object or `null`) — the user who set the current topic (in
  the same format as a user `"source"` field), or `null` if not reported
- `"topic_set_at"` (string, integer, or `null`) — the time at which the current
  topic was set as an RFC 3339 timestamp, or `null` if not reported
    - In the unlikely event that a timestamp cannot be constructed from the
      value reported by the server, the value of this field will be a UNIX
      timestamp (integer number of seconds since the UNIX epoch).
- `"channel_status"` (string) — the channel's status: one of `"Public"`,
  `"Secret"`, or `"Private"`
- `"users"` (list of objects) — The users currently joined to the channel along
  with their membership statuses therein.  Each object in the list contains the
  fields `"nickname"` (string) and `"membership"` (`"Founder"`, `"Protected"`,
  `"Operator"`, `"HalfOperator"`, `"Voiced"`, or `null`)

`"kick"` Event
----------------

Emitted when an `KICK` message is received.

JSON objects for this event type have the following additional fields:

- `"channel"` (string)
- `"users"` (list of strings)
- `"comment"` (string or `null`)

`"kill"` Event
----------------

Emitted when a `KILL` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional fields:

- `"nickname"` (string)
- `"comment"` (string)

`"links"` Event
----------------

Emitted when a `LINKS` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have no additional fields.

`"list"` Event
----------------

Emitted when a `LIST` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional fields:

- `"channels"` (list of strings) — the list of channel names in the "channels"
  parameter of the message
- `"elistconds"` (list of strings) — the list of `ELIST` conditions in the
  "elistconds" parameter of the message

`"lusers"` Event
----------------

Emitted when a `LUSERS` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have no additional fields.

`"mode"` Event
----------------

Emitted when a `MODE` message is received.

JSON objects for this event type have the following additional fields:

- `"target"` (string)
- `"modestring"` (string or `null`)
- `"arguments"` (list of strings)

`"motd"` Event
----------------

Emitted when a `MOTD` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional field:

- `"target"` (string or `null`)

`"names"` Event
----------------

Emitted when a `NAMES` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional field:

- `"channels"` (list of strings)

`"nick"` Event
----------------

Emitted when a `NICK` message is received.

JSON objects for this event type have the following additional field:

- `"nickname"` (string)

`"notice"` Event
----------------

Emitted when a `NOTICE` message is received.

JSON objects for this event type have the following additional fields:

- `"targets"` (list of strings)
- `"text"` (string)

`"oper"` Event
----------------

Emitted when an `OPER` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional fields:

- `"name"` (string)
- `"password"` (string)

`"parse_error"` Event
---------------------

Emitted when the program receives a message from the server that it cannot
parse.

JSON objects for this event type have the following additional fields:

- `"line"` (string) — the message in question, without the line ending
- `"error"` (string) — a human-readable description of the parse error that
  occurred and its causes

`"part"` Event
----------------

Emitted when a `PART` message is received.

JSON objects for this event type have the following additional fields:

- `"channels"` (list of strings)
- `"reason"` (string or `null`)

`"pass"` Event
----------------

Emitted when a `PASS` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional field:

- `"password"` (string)

`"pong"` Event
----------------

Emitted when a `PONG` message is received.

This event should not be emitted during normal operation and is included only
for completeness.

JSON objects for this event type have the following additional field:

- `"token"` (string)

`"ping"` Event
----------------

Emitted when a `PING` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional field:

- `"token"` (string)

`"privmsg"` Event
----------------

Emitted when a `PRIVMSG` message is received.

JSON objects for this event type have the following additional fields:

- `"targets"` (list of strings)
- `"text"` (string)

`"quit"` Event
----------------

Emitted when a `QUIT` message is received.

JSON objects for this event type have the following additional field:

- `"reason"` (string or `null`)

`"rehash"` Event
----------------

Emitted when a `REHASH` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have no additional fields.

`"reply"` Event
---------------

Emitted when a numeric reply message is received.

JSON objects for this event type have the following additional fields:

- `"code"` (integer) — the value of the reply's numeric code
- `"name"` (string or `null`) — the name of the reply (e.g., `"RPL_WELCOME"` or
  `"ERR_NOMOTD"`) or `null` if it is not known
- `"parameters"` (list of strings) — the parameters of the reply after the
  numeric code

`"restart"` Event
----------------

Emitted when a `RESTART` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have no additional fields.

`"squit"` Event
----------------

Emitted when an `SQUIT` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional fields:

- `"server"` (string)
- `"comment"` (string)

`"stats"` Event
----------------

Emitted when a `STATS` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional fields:

- `"query"` (string)
- `"server"` (string or `null`)

`"time"` Event
----------------

Emitted when a `TIME` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional field:

- `"server"` (string or `null`)

`"topic"` Event
----------------

Emitted when an `TOPIC` message is received.

JSON objects for this event type have the following additional fields:

- `"channel"` (string)
- `"topic"` (string or `null`)

`"user"` Event
----------------

Emitted when a `USER` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional fields:

- `"username"` (string)
- `"realname"` (string)

`"userhost"` Event
----------------

Emitted when a `USERHOST` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional field:

- `"nicknames"` (list of strings)

`"version"` Event
----------------

Emitted when a`VERSION` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional field:

- `"target"` (string or `null`)

`"wallops"` Event
----------------

Emitted when a `WALLOPS` message is received.

JSON objects for this event type have the following additional field:

- `"text"` (string)

`"who"` Event
----------------

Emitted when a `WHO` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional field:

- `"mask"` (string)

`"whois"` Event
----------------

Emitted when an `WHOIS` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional fields:

- `"target"` (string or `null`)
- `"nickname"` (string)

`"whowas"` Event
----------------

Emitted when an `WHOWAS` message is received.

This event should not occur during normal operation and is included only for
completeness.

JSON objects for this event type have the following additional fields:

- `"nickname"` (string)
- `"count"` (integer or `null`)
