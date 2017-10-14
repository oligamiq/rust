// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(non_camel_case_types, non_snake_case)]

//! Code that is useful in various trans modules.

use rustc::hir::def_id::DefId;
use rustc::hir::map::DefPathData;
use rustc::middle::lang_items::LangItem;
use monomorphize;
use rustc::traits;
use rustc::ty::{self, Ty, TyCtxt};
use rustc::ty::layout::{Layout, LayoutTyper};
use rustc::ty::subst::{Kind, Subst, Substs};
use rustc::hir;

use libc::{c_uint, c_char};
use std::iter;

use syntax::abi::Abi;
use syntax::attr;
use syntax::symbol::InternedString;
use syntax_pos::{Span, DUMMY_SP};

use context::{CrateContext, SharedCrateContext};

pub fn type_is_fat_ptr<'a, 'tcx>(ccx: &CrateContext<'a, 'tcx>, ty: Ty<'tcx>) -> bool {
    if let Layout::FatPointer { .. } = *ccx.layout_of(ty) {
        true
    } else {
        false
    }
}

pub fn type_is_immediate<'a, 'tcx>(ccx: &CrateContext<'a, 'tcx>, ty: Ty<'tcx>) -> bool {
    let layout = ccx.layout_of(ty);
    match *layout {
        Layout::CEnum { .. } |
        Layout::Scalar { .. } |
        Layout::Vector { .. } => true,

        Layout::FatPointer { .. } => false,

        Layout::Array { .. } |
        Layout::Univariant { .. } |
        Layout::General { .. } |
        Layout::UntaggedUnion { .. } |
        Layout::RawNullablePointer { .. } |
        Layout::StructWrappedNullablePointer { .. } => {
            !layout.is_unsized() && layout.size(ccx).bytes() == 0
        }
    }
}

/// Returns Some([a, b]) if the type has a pair of fields with types a and b.
pub fn type_pair_fields<'a, 'tcx>(ccx: &CrateContext<'a, 'tcx>, ty: Ty<'tcx>)
                                  -> Option<[Ty<'tcx>; 2]> {
    match ty.sty {
        ty::TyAdt(adt, substs) => {
            assert_eq!(adt.variants.len(), 1);
            let fields = &adt.variants[0].fields;
            if fields.len() != 2 {
                return None;
            }
            Some([monomorphize::field_ty(ccx.tcx(), substs, &fields[0]),
                  monomorphize::field_ty(ccx.tcx(), substs, &fields[1])])
        }
        ty::TyClosure(def_id, substs) => {
            let mut tys = substs.upvar_tys(def_id, ccx.tcx());
            tys.next().and_then(|first_ty| tys.next().and_then(|second_ty| {
                if tys.next().is_some() {
                    None
                } else {
                    Some([first_ty, second_ty])
                }
            }))
        }
        ty::TyGenerator(def_id, substs, _) => {
            let mut tys = substs.field_tys(def_id, ccx.tcx());
            tys.next().and_then(|first_ty| tys.next().and_then(|second_ty| {
                if tys.next().is_some() {
                    None
                } else {
                    Some([first_ty, second_ty])
                }
            }))
        }
        ty::TyTuple(tys, _) => {
            if tys.len() != 2 {
                return None;
            }
            Some([tys[0], tys[1]])
        }
        _ => None
    }
}

/// Returns true if the type is represented as a pair of immediates.
pub fn type_is_imm_pair<'a, 'tcx>(ccx: &CrateContext<'a, 'tcx>, ty: Ty<'tcx>)
                                  -> bool {
    match *ccx.layout_of(ty) {
        Layout::FatPointer { .. } => true,
        Layout::Univariant { ref variant, .. } => {
            // There must be only 2 fields.
            if variant.offsets.len() != 2 {
                return false;
            }

            match type_pair_fields(ccx, ty) {
                Some([a, b]) => {
                    type_is_immediate(ccx, a) && type_is_immediate(ccx, b)
                }
                None => false
            }
        }
        _ => false
    }
}

/// Identify types which have size zero at runtime.
pub fn type_is_zero_size<'a, 'tcx>(ccx: &CrateContext<'a, 'tcx>, ty: Ty<'tcx>) -> bool {
    let layout = ccx.layout_of(ty);
    !layout.is_unsized() && layout.size(ccx).bytes() == 0
}

pub fn type_needs_drop<'a, 'tcx>(tcx: TyCtxt<'a, 'tcx, 'tcx>, ty: Ty<'tcx>) -> bool {
    ty.needs_drop(tcx, ty::ParamEnv::empty(traits::Reveal::All))
}

pub fn type_is_sized<'a, 'tcx>(tcx: TyCtxt<'a, 'tcx, 'tcx>, ty: Ty<'tcx>) -> bool {
    ty.is_sized(tcx, ty::ParamEnv::empty(traits::Reveal::All), DUMMY_SP)
}

pub fn type_is_freeze<'a, 'tcx>(tcx: TyCtxt<'a, 'tcx, 'tcx>, ty: Ty<'tcx>) -> bool {
    ty.is_freeze(tcx, ty::ParamEnv::empty(traits::Reveal::All), DUMMY_SP)
}

#[inline]
fn hi_lo_to_u128(lo: u64, hi: u64) -> u128 {
    ((hi as u128) << 64) | (lo as u128)
}

pub fn langcall(tcx: TyCtxt,
                span: Option<Span>,
                msg: &str,
                li: LangItem)
                -> DefId {
    match tcx.lang_items().require(li) {
        Ok(id) => id,
        Err(s) => {
            let msg = format!("{} {}", msg, s);
            match span {
                Some(span) => tcx.sess.span_fatal(span, &msg[..]),
                None => tcx.sess.fatal(&msg[..]),
            }
        }
    }
}

pub fn ty_fn_sig<'a, 'tcx>(ccx: &CrateContext<'a, 'tcx>,
                           ty: Ty<'tcx>)
                           -> ty::PolyFnSig<'tcx>
{
    match ty.sty {
        ty::TyFnDef(..) |
        // Shims currently have type TyFnPtr. Not sure this should remain.
        ty::TyFnPtr(_) => ty.fn_sig(ccx.tcx()),
        ty::TyClosure(def_id, substs) => {
            let tcx = ccx.tcx();
            let sig = tcx.fn_sig(def_id).subst(tcx, substs.substs);

            let env_region = ty::ReLateBound(ty::DebruijnIndex::new(1), ty::BrEnv);
            let env_ty = match tcx.closure_kind(def_id) {
                ty::ClosureKind::Fn => tcx.mk_imm_ref(tcx.mk_region(env_region), ty),
                ty::ClosureKind::FnMut => tcx.mk_mut_ref(tcx.mk_region(env_region), ty),
                ty::ClosureKind::FnOnce => ty,
            };

            sig.map_bound(|sig| tcx.mk_fn_sig(
                iter::once(env_ty).chain(sig.inputs().iter().cloned()),
                sig.output(),
                sig.variadic,
                sig.unsafety,
                sig.abi
            ))
        }
        ty::TyGenerator(def_id, substs, _) => {
            let tcx = ccx.tcx();
            let sig = tcx.generator_sig(def_id).unwrap().subst(tcx, substs.substs);

            let env_region = ty::ReLateBound(ty::DebruijnIndex::new(1), ty::BrEnv);
            let env_ty = tcx.mk_mut_ref(tcx.mk_region(env_region), ty);

            sig.map_bound(|sig| {
                let state_did = tcx.lang_items().gen_state().unwrap();
                let state_adt_ref = tcx.adt_def(state_did);
                let state_substs = tcx.mk_substs([Kind::from(sig.yield_ty),
                    Kind::from(sig.return_ty)].iter());
                let ret_ty = tcx.mk_adt(state_adt_ref, state_substs);

                tcx.mk_fn_sig(iter::once(env_ty),
                    ret_ty,
                    false,
                    hir::Unsafety::Normal,
                    Abi::Rust
                )
            })
        }
        _ => bug!("unexpected type {:?} to ty_fn_sig", ty)
    }
}

pub fn requests_inline<'a, 'tcx>(
    tcx: TyCtxt<'a, 'tcx, 'tcx>,
    instance: &ty::Instance<'tcx>
) -> bool {
    if is_inline_instance(tcx, instance) {
        return true
    }
    if let ty::InstanceDef::DropGlue(..) = instance.def {
        // Drop glue wants to be instantiated at every translation
        // unit, but without an #[inline] hint. We should make this
        // available to normal end-users.
        return true
    }
    attr::requests_inline(&instance.def.attrs(tcx)[..])
}

pub fn is_inline_instance<'a, 'tcx>(
    tcx: TyCtxt<'a, 'tcx, 'tcx>,
    instance: &ty::Instance<'tcx>
) -> bool {
    let def_id = match instance.def {
        ty::InstanceDef::Item(def_id) => def_id,
        ty::InstanceDef::DropGlue(_, Some(_)) => return false,
        _ => return true
    };
    match tcx.def_key(def_id).disambiguated_data.data {
        DefPathData::StructCtor |
        DefPathData::EnumVariant(..) |
        DefPathData::ClosureExpr => true,
        _ => false
    }
}

/// Given a DefId and some Substs, produces the monomorphic item type.
pub fn def_ty<'a, 'tcx>(tcx: TyCtxt<'a, 'tcx, 'tcx>,
                        def_id: DefId,
                        substs: &'tcx Substs<'tcx>)
                        -> Ty<'tcx>
{
    let ty = tcx.type_of(def_id);
    tcx.trans_apply_param_substs(substs, &ty)
}

/// Return the substituted type of an instance.
pub fn instance_ty<'a, 'tcx>(tcx: TyCtxt<'a, 'tcx, 'tcx>,
                             instance: &ty::Instance<'tcx>)
                             -> Ty<'tcx>
{
    let ty = instance.def.def_ty(tcx);
    tcx.trans_apply_param_substs(instance.substs, &ty)
}
