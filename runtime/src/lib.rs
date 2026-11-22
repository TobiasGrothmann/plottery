#![doc = include_str!("../README.md")]

// Lets generated code use `::plottery_runtime::` paths inside this crate as well.
extern crate self as plottery_runtime;

mod project_main;
mod project_params;
mod protocol;
mod run_stream;

pub use plottery_runtime_macros::*;
pub use project_main::*;
pub use project_params::*;
pub use protocol::PROTOCOL_VERSION;
pub use run_stream::MessageKind;
