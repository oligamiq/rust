#![doc(html_root_url = "https://doc.rust-lang.org/nightly/nightly-rustc/")]
#![recursion_limit = "256"]

//! This crate contains codegen code that is used by all codegen backends (LLVM and others).
//! The backend-agnostic functions of this crate use functions defined in various traits that
//! have to be implemented by each backends.

#[macro_use]
extern crate rustc_macros;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate rustc_middle;

use rustc_ast as ast;
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_data_structures::sync::Lrc;
use rustc_hir::def_id::CrateNum;
use rustc_middle::dep_graph::WorkProduct;
use rustc_middle::middle::dependency_format::Dependencies;
use rustc_middle::middle::exported_symbols::SymbolExportKind;
use rustc_middle::ty::query::{ExternProviders, Providers};
use rustc_serialize::opaque::{MemDecoder, MemEncoder};
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use rustc_session::config::{CrateType, OutputFilenames, OutputType, RUST_CGU_EXT};
use rustc_session::cstore::{self, CrateSource};
use rustc_session::utils::NativeLibKind;
use rustc_span::symbol::Symbol;
use rustc_span::DebuggerVisualizerFile;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub mod builder;
pub mod discriminant;
