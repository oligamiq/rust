// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use rustc_target::abi::call::FnType;

use traits::*;

use rustc::ty::Ty;

#[derive(Copy, Clone, Debug)]
pub struct VirtualIndex(u64);

pub const DESTRUCTOR: VirtualIndex = VirtualIndex(0);
pub const SIZE: VirtualIndex = VirtualIndex(1);
pub const ALIGN: VirtualIndex = VirtualIndex(2);

impl<'a, 'tcx: 'a> VirtualIndex {
    pub fn from_index(index: usize) -> Self {
        VirtualIndex(index as u64 + 3)
    }

    pub fn get_fn<Bx: BuilderMethods<'a, 'tcx>>(
        self,
        bx: &mut Bx,
        llvtable: Bx::Value,
        fn_ty: &FnType<'tcx, Ty<'tcx>>
    ) -> Bx::Value {
        // Load the data pointer from the object.
        debug!("get_fn({:?}, {:?})", llvtable, self);

        let llvtable = bx.pointercast(
            llvtable,
            bx.cx().type_ptr_to(bx.cx().fn_ptr_backend_type(fn_ty))
        );
        let ptr_align = bx.tcx().data_layout.pointer_align.abi;
        let gep = bx.inbounds_gep(llvtable, &[bx.cx().const_usize(self.0)]);
        let ptr = bx.load(gep, ptr_align);
        bx.nonnull_metadata(ptr);
        // Vtable loads are invariant
        bx.set_invariant_load(ptr);
        ptr
    }

    pub fn get_usize<Bx: BuilderMethods<'a, 'tcx>>(
        self,
        bx: &mut Bx,
        llvtable: Bx::Value
    ) -> Bx::Value {
        // Load the data pointer from the object.
        debug!("get_int({:?}, {:?})", llvtable, self);

        let llvtable = bx.pointercast(llvtable, bx.cx().type_ptr_to(bx.cx().type_isize()));
        let usize_align = bx.tcx().data_layout.pointer_align.abi;
        let gep = bx.inbounds_gep(llvtable, &[bx.cx().const_usize(self.0)]);
        let ptr = bx.load(gep, usize_align);
        // Vtable loads are invariant
        bx.set_invariant_load(ptr);
        ptr
    }
}
