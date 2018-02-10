use rustc_data_structures::indexed_vec::IndexVec;
use rustc::ty::{self, TyCtxt, Ty, TypeFoldable, Instance, ParamTy};
use rustc::ty::fold::TypeFolder;
use rustc::ty::subst::{Kind, UnpackedKind};
use rustc::mir::Promoted;
use rustc::mir::visit::{Visitor, TyContext};

/// Replace substs which arent used by the function with TyError,
/// so that it doesnt end up in the binary multiple times
pub(crate) fn collapse_interchangable_instances<'a, 'tcx>(tcx: TyCtxt<'a, 'tcx, 'tcx>, mut inst: Instance<'tcx>) -> Instance<'tcx> {
    info!("replace_unused_substs_with_ty_error({:?})", inst);

    if inst.substs.is_noop() || !tcx.is_mir_available(inst.def_id()) {
        return inst;
    }
    match inst.ty(tcx).sty {
        ty::TyFnDef(def_id, _) => {
            //let attrs = tcx.item_attrs(def_id);
            if tcx.lang_items().items().iter().find(|l|**l == Some(def_id)).is_some() {
                return inst; // Lang items dont work otherwise
            }
        }
        _ => return inst, // Closures dont work otherwise
    }

    let used_substs = used_substs_for_instance(tcx, inst);
    inst.substs = tcx._intern_substs(&inst.substs.into_iter().enumerate().map(|(i, subst)| {
        if let UnpackedKind::Type(ty) = subst.unpack() {
            let ty = if used_substs.substs.iter().find(|p|p.idx == i as u32).is_some() {
                ty.into()
            } else if let ty::TyParam(ref _param) = ty.sty { // Dont replace <closure_kind> and other internal params
                if false /*param.name.as_str().starts_with("<")*/ {
                    ty.into()
                } else {
                    tcx.mk_ty(ty::TyNever)
                }
            } else {
                tcx.mk_ty(ty::TyNever) // Can't use TyError as it gives some ICE in rustc_trans::callee::get_fn
            };
            Kind::from(ty)
        } else {
            (*subst).clone()
        }
    }).collect::<Vec<_>>());
    info!("replace_unused_substs_with_ty_error(_) -> {:?}", inst);
    inst
}

#[derive(Debug, Default, Clone)]
pub struct UsedSubsts {
    pub substs: Vec<ParamTy>,
    pub promoted: IndexVec<Promoted, UsedSubsts>,
}

impl_stable_hash_for! { struct UsedSubsts { substs, promoted } }

fn used_substs_for_instance<'a, 'tcx: 'a>(tcx: TyCtxt<'a ,'tcx, 'tcx>, instance: Instance<'tcx>) -> UsedSubsts {
    struct SubstsVisitor<'a, 'gcx: 'a + 'tcx, 'tcx: 'a>(TyCtxt<'a, 'gcx, 'tcx>, UsedSubsts);

    impl<'a, 'gcx: 'a + 'tcx, 'tcx: 'a> Visitor<'tcx> for SubstsVisitor<'a, 'gcx, 'tcx> {
        fn visit_ty(&mut self, ty: &Ty<'tcx>, _: TyContext) {
            self.fold_ty(ty);
        }
    }

    impl<'a, 'gcx: 'a + 'tcx, 'tcx: 'a> TypeFolder<'gcx, 'tcx> for SubstsVisitor<'a, 'gcx, 'tcx> {
        fn tcx<'b>(&'b self) -> TyCtxt<'b, 'gcx, 'tcx> {
            self.0
        }
        fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
            if !ty.needs_subst() {
                return ty;
            }
            match ty.sty {
                ty::TyParam(param) => {
                    self.1.substs.push(param);
                    ty
                }
                ty::TyFnDef(_, substs) => {
                    for subst in substs {
                        if let UnpackedKind::Type(ty) = subst.unpack() {
                            ty.fold_with(self);
                        }
                    }
                    ty.super_fold_with(self)
                }
                ty::TyClosure(_, closure_substs) => {
                    for subst in closure_substs.substs {
                        if let UnpackedKind::Type(ty) = subst.unpack() {
                            ty.fold_with(self);
                        }
                    }
                    ty.super_fold_with(self)
                }
                _ => ty.super_fold_with(self)
            }
        }
    }

    let mir = tcx.instance_mir(instance.def);
    let sig = ::rustc::ty::ty_fn_sig(tcx, instance.ty(tcx));
    let sig = tcx.normalize_erasing_late_bound_regions(ty::ParamEnv::reveal_all(), &sig);
    let mut substs_visitor = SubstsVisitor(tcx, UsedSubsts::default());
    substs_visitor.visit_mir(mir);
    for ty in sig.inputs().iter() {
        ty.fold_with(&mut substs_visitor);
    }
    sig.output().fold_with(&mut substs_visitor);
    let mut used_substs = substs_visitor.1;
    used_substs.substs.sort_by_key(|s|s.idx);
    used_substs.substs.dedup_by_key(|s|s.idx);
    used_substs
}
