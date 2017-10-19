// Copyright 2012-2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The Rust compiler.
//!
//! # Note
//!
//! This API is completely unstable and subject to change.

#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
      html_favicon_url = "https://doc.rust-lang.org/favicon.ico",
      html_root_url = "https://doc.rust-lang.org/nightly/")]
//#![deny(warnings)]
#![allow(warnings)]

#![feature(box_patterns)]
#![feature(box_syntax)]
#![feature(custom_attribute)]
#![allow(unused_attributes)]
#![feature(i128_type)]
#![feature(libc)]
#![feature(quote)]
#![feature(rustc_diagnostic_macros)]
#![feature(slice_patterns)]
#![feature(conservative_impl_trait)]

#![cfg_attr(stage0, feature(const_fn))]
#![cfg_attr(not(stage0), feature(const_atomic_bool_new))]
#![cfg_attr(not(stage0), feature(const_once_new))]

use rustc::dep_graph::WorkProduct;
use syntax_pos::symbol::Symbol;

extern crate cretonne;
extern crate cton_frontend;
#[macro_use]
extern crate bitflags;
extern crate flate2;
extern crate libc;
extern crate owning_ref;
#[macro_use] extern crate rustc;
extern crate rustc_allocator;
extern crate rustc_back;
extern crate rustc_data_structures;
extern crate rustc_incremental;
extern crate rustc_const_math;
extern crate rustc_trans_utils;
extern crate rustc_demangle;
extern crate jobserver;
extern crate num_cpus;

#[macro_use] extern crate log;
#[macro_use] extern crate syntax;
extern crate syntax_pos;
extern crate rustc_errors as errors;
extern crate serialize;
#[cfg(windows)]
extern crate cc; // Used to locate MSVC

//mod collector;
mod context;
//mod common;
//mod trans_item;
//mod monomorphize;
mod trans_crate;

use std::any::Any;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;

use rustc::dep_graph::DepGraph;
use rustc::hir::def_id::CrateNum;
use rustc::middle::cstore::MetadataLoader;
use rustc::middle::cstore::{NativeLibrary, CrateSource, LibSource};
use rustc::session::Session;
use rustc::session::config::{OutputFilenames, OutputType};
use rustc::ty::maps::Providers;
use rustc::ty::{self, TyCtxt};
use rustc::util::nodemap::{FxHashSet, FxHashMap};

pub struct CretonneTransCrate(());

impl CretonneTransCrate {
    pub fn new() -> Self {
        CretonneTransCrate(())
    }
}

impl rustc_trans_utils::trans_crate::TransCrate for CretonneTransCrate {
    type MetadataLoader = rustc_trans_utils::trans_crate::NoLlvmMetadataLoader;
    type OngoingCrateTranslation = ();
    type TranslatedCrate = ();

    fn metadata_loader() -> Box<MetadataLoader> {
        box rustc_trans_utils::trans_crate::NoLlvmMetadataLoader
    }

    fn provide_local(providers: &mut ty::maps::Providers) {
        provide_local(providers);
    }

    fn provide_extern(providers: &mut ty::maps::Providers) {
        provide_extern(providers);
    }

    fn trans_crate<'a, 'tcx>(
        tcx: TyCtxt<'a, 'tcx, 'tcx>,
        rx: mpsc::Receiver<Box<Any + Send>>
    ) -> Self::OngoingCrateTranslation {
        trans_crate::trans_crate(tcx)
    }

    fn join_trans(
        trans: Self::OngoingCrateTranslation,
        sess: &Session,
        dep_graph: &DepGraph
    ) -> Self::TranslatedCrate {
        
    }

    fn link_binary(sess: &Session, trans: &Self::TranslatedCrate, outputs: &OutputFilenames) {
        panic!("No linking support yet");
    }

    fn dump_incremental_data(trans: &Self::TranslatedCrate) {
        unimplemented!();
    }
}

__build_diagnostic_array! { librustc_trans, DIAGNOSTICS }

fn provide_local(providers: &mut Providers) {}

fn provide_extern(providers: &mut Providers) {}
