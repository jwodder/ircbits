#![cfg_attr(docsrs, feature(doc_cfg))]
//! `irctext` is a Rust library for working with IRC messages (parsing,
//! constructing, rendering, etc.) in which every type of message (both client
//! messages and replies) is represented by a dedicated type that only permits
//! values that conform to the specification at <https://modern.ircdocs.horse>.
//!
//! Features
//! ========
//!
//! The `irctext` crate has the following optional features:
//!
//! - `anstyle` — Enables converting formatting types to [`anstyle`] types and
//!   rendering IRC-styled text with ANSI sequences
//!
//! - `serde` — Enables serializing & deserializing most types with
//!   [`serde`]

#[macro_use]
mod validstr;

pub mod clientmsgs;
mod command;
mod consts;
pub mod ctcp;
pub mod formatting;
mod message;
mod parameters;
mod raw_message;
pub mod replies;
mod source;
pub mod types;
mod util;
mod verb;
pub use crate::clientmsgs::{ClientMessage, ClientMessageError, ClientMessageParts};
pub use crate::command::*;
pub use crate::consts::*;
pub use crate::message::*;
pub use crate::parameters::*;
pub use crate::raw_message::*;
pub use crate::replies::{Reply, ReplyError, ReplyParts};
pub use crate::source::*;
pub use crate::validstr::TryFromStringError;
pub use crate::verb::*;
