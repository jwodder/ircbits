Per-Package IRCv3 Extension Support
===================================

Draft/work-in-progress extensions and deprecated extensions are not listed
here.

| Extension                                                     | `irctext` | `ircnet` | `echobot` | `ircevents` | `ircprobe` | `ircwatch` | `listchans` | `msgtimes` |
| ------------------------------------------------------------- | :-------: | :------: | :-------: | :---------: | :--------: | :--------: | :---------: | :--------: |
| [`account-extban`][]                                          | ✓         | ~        | ~         | ~           | ~          | ~          | ~           | ~          |
| [`account-notify`][]                                          | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [`account-tag`][]                                             | ~         | ~        | ~         | ~           | ~          | ~          | ~           | ~          |
| [`away-notify`[]                                              | ✓         | ~        | ~         | ~           | ~          | ~          | ~           | ~          |
| [`batch`][]                                                   | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [Bot Mode][]                                                  | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [Capability Negotiation][], version 302                       | ✓         | ✓        | ✓         | ✓           | ✓          | ✓          | ✓           | ✓          |
| [`chathistory` batch type][]                                  | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [`chghost`][]                                                 | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [`echo-message`][]                                            | ~         | ~        | ✗         | ~           | ~          | ~          | ~           | ~          |
| [`extended-join`][]                                           | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [Extended Monitor][]                                          | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [`invite-notify`][]                                           | ✓         | ~        | ~         | ✓           | ~          | ✓          | ~           | ~          |
| [`labeled-response`][]                                        | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [`message-ids`][]                                             | ~         | ~        | ~         | ✓           | ~          | ~          | ~           | ~          |
| [Message Tags][]                                              | ✓         | ~        | ~         | ~           | ~          | ~          | ~           | ~          |
| [Monitor][]                                                   | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [`multi-prefix`][]                                            | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [`netsplit` and `netjoin` batch types][]                      | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| SASL Authentication, versions [3.1][sasl31] and [3.2][sasl32] | ✓         | ✓ (1)    | ✓ (1)     | ✓ (1)       | ✓ (1)      | ✓ (1)      | ✓ (1)       | ✓ (1)      |
| [`server-time`][]                                             | ~         | ~        | ~         | ✓           | ~          | ~          | ~           | ~          |
| [`setname`][]                                                 | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [Standard Replies][]                                          | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [`sts`][]                                                     | ~         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [`typing` client-only tag][]                                  | ~         | ~        | ~         | ✓           | ~          | ✓          | ~           | ~          |
| [`userhost-in-names`][]                                       | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [`UTF8ONLY`][]                                                | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [`WEBIRC`][]                                                  | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |
| [`WHOX`][]                                                    | ✗         | ✗        | ✗         | ✗           | ✗          | ✗          | ✗           | ✗          |

[Capability Negotation]: https://ircv3.net/specs/extensions/capability-negotiation
[Message Tags]: https://ircv3.net/specs/extensions/message-tags
[sasl31]: https://ircv3.net/specs/extensions/sasl-3.1
[sasl32]: https://ircv3.net/specs/extensions/sasl-3.2
[`account-extban`]: https://ircv3.net/specs/extensions/account-extban
[`account-notify`]: https://ircv3.net/specs/extensions/account-notify
[`account-tag`]: https://ircv3.net/specs/extensions/account-tag
[`extended-join`]: https://ircv3.net/specs/extensions/extended-join
[`away-notify`]: https://ircv3.net/specs/extensions/away-notify
[`batch`]: https://ircv3.net/specs/extensions/batch
[`netsplit` and `netjoin` batch types]: https://ircv3.net/specs/batches/netsplit
[`chathistory` batch type]: https://ircv3.net/specs/batches/chathistory
[Bot Mode]: https://ircv3.net/specs/extensions/bot-mode
[`chghost`]: https://ircv3.net/specs/extensions/chghost
[`setname`]: https://ircv3.net/specs/extensions/setname
[`typing` client-only tag]: https://ircv3.net/specs/client-tags/typing
[`echo-message`]: https://ircv3.net/specs/extensions/echo-message
[`invite-notify`]: https://ircv3.net/specs/extensions/invite-notify
[`labeled-response`]: https://ircv3.net/specs/extensions/labeled-response
[`multi-prefix`]: https://ircv3.net/specs/extensions/multi-prefix
[`userhost-in-names`]: https://ircv3.net/specs/extensions/userhost-in-names
[`WHOX`]: https://ircv3.net/specs/extensions/whox
[`message-ids`]: https://ircv3.net/specs/extensions/message-ids
[Monitor]: https://ircv3.net/specs/extensions/monitor
[Extended Monitor]: https://ircv3.net/specs/extensions/extended-monitor
[`server-time`]: https://ircv3.net/specs/extensions/server-time
[Standard Replies]: https://ircv3.net/specs/extensions/standard-replies
[`sts`]: https://ircv3.net/specs/extensions/sts
[`UTF8ONLY`]: https://ircv3.net/specs/extensions/utf8-only
[`WEBIRC`]: https://ircv3.net/specs/extensions/webirc

Key:

- ✓ — Supported and can make use of
- ~ — Compatible with/agnostic to; can function when the feature is enabled but
  does not make use of it
- ✗ — Not supported; enabling the feature may cause something to break

Notes:

- (1) — `PLAIN` mechanism only
