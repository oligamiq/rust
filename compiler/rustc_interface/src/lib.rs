#![feature(box_patterns)]
#![feature(decl_macro)]
#![feature(internal_output_capture)]
#![feature(thread_spawn_unchecked)]
#![feature(lazy_cell)]
#![feature(let_chains)]
#![feature(try_blocks)]
#![recursion_limit = "256"]
#![allow(rustc::potential_query_instability)]
#![deny(rustc::untranslatable_diagnostic)]
#![deny(rustc::diagnostic_outside_of_impl)]

#[macro_use]
extern crate tracing;

mod callbacks;
mod errors;
pub mod interface;
mod passes;
mod proc_macro_decls;
mod queries;
pub mod util;

pub use callbacks::setup_callbacks;
pub use interface::{run_compiler, Config};
pub use passes::{create_global_ctxt, write_dep_info, DEFAULT_QUERY_PROVIDERS};
pub use queries::{Linker, Queries};

#[cfg(test)]
mod tests;

rustc_fluent_macro::fluent_messages! { "../messages.ftl" }
